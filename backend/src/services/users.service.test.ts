import { describe, expect, it, vi } from 'vitest';
import { NotFoundError } from '../errors.js';
import { UsersService } from './users.service.js';

const user = { id: '9c4b9a62-7c4e-4ee9-88af-6f4c07637965', email: 'ada@example.com', name: 'Ada', createdAt: new Date(), updatedAt: new Date() };

describe('UsersService', () => {
  it('creates and lists users through the ORM delegate', async () => {
    const users = { create: vi.fn().mockResolvedValue(user), findMany: vi.fn().mockResolvedValue([user]), findUnique: vi.fn(), update: vi.fn(), delete: vi.fn() };
    const service = new UsersService(users as never);
    await expect(service.create({ email: user.email, name: user.name })).resolves.toEqual(user);
    await expect(service.list()).resolves.toEqual([user]);
    expect(users.findMany).toHaveBeenCalledWith({ orderBy: { createdAt: 'desc' } });
  });

  it('raises not found when the record does not exist', async () => {
    const users = { create: vi.fn(), findMany: vi.fn(), findUnique: vi.fn().mockResolvedValue(null), update: vi.fn(), delete: vi.fn() };
    await expect(new UsersService(users as never).get(user.id)).rejects.toBeInstanceOf(NotFoundError);
  });
});
