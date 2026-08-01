import { describe, expect, it } from 'vitest';
import { parseEnvironment } from './env.js';

const validEnvironment = {
  DATABASE_URL: 'postgresql://user:password@localhost:5432/tap',
  REDIS_URL: 'redis://localhost:6379',
  OPENSEARCH_NODE: 'http://localhost:9200',
  OPENSEARCH_USERNAME: 'admin',
  OPENSEARCH_PASSWORD: 'admin',
};

describe('parseEnvironment', () => {
  it('applies development defaults', () => {
    expect(parseEnvironment(validEnvironment)).toMatchObject({
      PORT: 3000,
      NODE_ENV: 'development',
      CORS_ORIGIN: 'http://localhost:4200',
    });
  });

  it('rejects invalid required configuration', () => {
    expect(() => parseEnvironment({ ...validEnvironment, REDIS_URL: 'not-a-url' })).toThrow();
  });
});
