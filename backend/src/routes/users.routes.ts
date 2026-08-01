import { Router } from 'express';
import { z } from 'zod';
import { ValidationError } from '../errors.js';
import type { UsersService } from '../services/users.service.js';

const userIdSchema = z.object({ id: z.string().uuid() });
const createUserSchema = z.object({
  email: z.string().trim().email(),
  name: z.string().trim().min(1).max(200),
});
const updateUserSchema = createUserSchema.partial().refine((value) => Object.keys(value).length > 0, {
  message: 'At least one field must be provided',
});

function parse<T>(schema: z.ZodType<T>, value: unknown): T {
  const result = schema.safeParse(value);
  if (!result.success) throw new ValidationError(result.error.flatten());
  return result.data;
}

export function createUsersRouter(users: UsersService): Router {
  const router = Router();

  router.get('/', async (_request, response, next) => {
    try {
      response.json(await users.list());
    } catch (error) { next(error); }
  });

  router.post('/', async (request, response, next) => {
    try {
      response.status(201).json(await users.create(parse(createUserSchema, request.body)));
    } catch (error) { next(error); }
  });

  router.get('/:id', async (request, response, next) => {
    try {
      response.json(await users.get(parse(userIdSchema, request.params).id));
    } catch (error) { next(error); }
  });

  router.patch('/:id', async (request, response, next) => {
    try {
      const { id } = parse(userIdSchema, request.params);
      response.json(await users.update(id, parse(updateUserSchema, request.body)));
    } catch (error) { next(error); }
  });

  router.delete('/:id', async (request, response, next) => {
    try {
      await users.remove(parse(userIdSchema, request.params).id);
      response.status(204).send();
    } catch (error) { next(error); }
  });

  return router;
}
