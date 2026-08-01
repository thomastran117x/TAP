import { createApp } from './app.js';
import { parseEnvironment } from './config/env.js';
import { createOpenSearchClient } from './lib/opensearch.js';
import { prisma } from './lib/prisma.js';
import { createRedisClient } from './lib/redis.js';
import { UsersService } from './services/users.service.js';

const environment = parseEnvironment();
const redis = createRedisClient(environment.REDIS_URL);
redis.on('error', (error) => console.error('Redis error', error));
const opensearch = createOpenSearchClient(
  environment.OPENSEARCH_NODE,
  environment.OPENSEARCH_USERNAME,
  environment.OPENSEARCH_PASSWORD,
);

const app = createApp(environment, {
  users: new UsersService(prisma.user),
  readiness: { database: prisma, redis, opensearch },
});

const server = app.listen(environment.PORT, () => {
  console.log(`Backend listening on http://localhost:${environment.PORT}`);
});

async function shutdown() {
  server.close();
  if (redis.isOpen) await redis.quit();
  await prisma.$disconnect();
}

process.once('SIGINT', shutdown);
process.once('SIGTERM', shutdown);
