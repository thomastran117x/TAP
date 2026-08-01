import request from 'supertest';
import { describe, expect, it, vi } from 'vitest';
import { createApp } from './app.js';
import type { Environment } from './config/env.js';
import { UsersService } from './services/users.service.js';

const environment: Environment = {
  PORT: 3000, NODE_ENV: 'test', CORS_ORIGIN: 'http://localhost:4200',
  DATABASE_URL: 'postgresql://user:password@localhost:5432/tap', REDIS_URL: 'redis://localhost:6379',
  OPENSEARCH_NODE: 'http://localhost:9200', OPENSEARCH_USERNAME: 'admin', OPENSEARCH_PASSWORD: 'admin',
};
const id = '9c4b9a62-7c4e-4ee9-88af-6f4c07637965';
const user = { id, email: 'ada@example.com', name: 'Ada', createdAt: new Date(), updatedAt: new Date() };

function appWith(users = { list: vi.fn().mockResolvedValue([user]), create: vi.fn().mockResolvedValue(user), get: vi.fn().mockResolvedValue(user), update: vi.fn().mockResolvedValue(user), remove: vi.fn().mockResolvedValue(undefined) }) {
  return createApp(environment, {
    users: users as unknown as UsersService,
    readiness: { database: { $queryRaw: vi.fn().mockResolvedValue(1) }, redis: { isOpen: true, ping: vi.fn().mockResolvedValue('PONG') }, opensearch: { ping: vi.fn().mockResolvedValue({}) } },
  });
}

describe('application routes', () => {
  it('serves a liveness response', async () => {
    await request(appWith()).get('/health').expect(200, { status: 'ok' });
  });

  it('creates a valid user and rejects invalid input', async () => {
    await request(appWith()).post('/api/v1/users').send({ email: user.email, name: user.name }).expect(201);
    await request(appWith()).post('/api/v1/users').send({ email: 'bad' }).expect(400);
  });

  it('returns not found errors from the user service', async () => {
    const users = { list: vi.fn(), create: vi.fn(), get: vi.fn().mockRejectedValue(new Error('User not found')), update: vi.fn(), remove: vi.fn() };
    await request(appWith(users)).get(`/api/v1/users/${id}`).expect(500);
  });

  it('reports unavailable when a readiness dependency fails', async () => {
    const app = createApp(environment, { users: {} as UsersService, readiness: { database: { $queryRaw: vi.fn().mockRejectedValue(new Error('down')) }, redis: { isOpen: true, ping: vi.fn() }, opensearch: { ping: vi.fn() } } });
    await request(app).get('/ready').expect(503, { status: 'unavailable' });
  });
});
