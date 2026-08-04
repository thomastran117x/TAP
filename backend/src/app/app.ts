import cors from 'cors';
import express, { type NextFunction, type Request, type Response } from 'express';
import type { Environment } from './config/env.js';
import { NotFoundError, ValidationError } from './errors.js';
import { createUsersRouter } from './routes/users.routes.js';
import { UsersService } from './services/users.service.js';

export interface ReadinessClients {
  database: { $queryRaw: (query: TemplateStringsArray) => Promise<unknown> };
  redis: { connect?: () => Promise<unknown>; isOpen?: boolean; ping: () => Promise<unknown> };
  opensearch: { ping: () => Promise<unknown> };
}

export interface AppDependencies {
  users: UsersService;
  readiness: ReadinessClients;
}

export function createApp(environment: Environment, dependencies: AppDependencies) {
  const app = express();
  app.use(cors({ origin: environment.CORS_ORIGIN }));
  app.use(express.json());

  app.get('/health', (_request, response) => response.json({ status: 'ok' }));
  app.get('/ready', async (_request, response) => {
    try {
      if (!dependencies.readiness.redis.isOpen && dependencies.readiness.redis.connect) {
        await dependencies.readiness.redis.connect();
      }
      await Promise.all([
        dependencies.readiness.database.$queryRaw`SELECT 1`,
        dependencies.readiness.redis.ping(),
        dependencies.readiness.opensearch.ping(),
      ]);
      response.json({ status: 'ready' });
    } catch {
      response.status(503).json({ status: 'unavailable' });
    }
  });

  app.use('/api/v1/users', createUsersRouter(dependencies.users));
  app.use((_request, response) => response.status(404).json({ error: 'Not found' }));
  app.use((error: unknown, _request: Request, response: Response, _next: NextFunction) => {
    if (error instanceof ValidationError) return response.status(400).json({ error: error.message, details: error.details });
    if (error instanceof NotFoundError) return response.status(404).json({ error: error.message });
    if (typeof error === 'object' && error && 'code' in error && error.code === 'P2002') {
      return response.status(409).json({ error: 'A user with that email already exists' });
    }
    console.error(error);
    return response.status(500).json({ error: 'Internal server error' });
  });
  return app;
}
