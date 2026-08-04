import { z } from 'zod';

const environmentSchema = z.object({
  PORT: z.coerce.number().int().positive().default(3000),
  NODE_ENV: z.enum(['development', 'test', 'production']).default('development'),
  DATABASE_URL: z.string().url(),
  REDIS_URL: z.string().url(),
  OPENSEARCH_NODE: z.string().url(),
  OPENSEARCH_USERNAME: z.string().min(1),
  OPENSEARCH_PASSWORD: z.string().min(1),
  CORS_ORIGIN: z.string().url().default('http://localhost:4200'),
});

export type Environment = z.infer<typeof environmentSchema>;

export function parseEnvironment(input: NodeJS.ProcessEnv = process.env): Environment {
  return environmentSchema.parse(input);
}
