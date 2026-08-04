import { createApp } from './app/app.js';
import { parseEnvironment } from './app/config/env.js';
import { createOpenSearchClient } from './app/lib/opensearch.js';
import { prisma } from './app/lib/prisma.js';
import { createRedisClient } from './app/lib/redis.js';
import { UsersService } from './app/services/users.service.js';

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
