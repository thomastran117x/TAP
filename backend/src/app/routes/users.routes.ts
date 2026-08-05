import { Router } from 'express';
import { createUsersController } from '../controllers/users.controller.js';
import type { UsersService } from '../services/users.service.js';

export function createUsersRouter(users: UsersService): Router {
  const router = Router();
  const controller = createUsersController(users);

  router.get('/', controller.list);
  router.post('/', controller.create);
  router.get('/:id', controller.get);
  router.patch('/:id', controller.update);
  router.delete('/:id', controller.remove);

  return router;
}