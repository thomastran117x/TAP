import type { Prisma, PrismaClient, User } from '@prisma/client';
import { NotFoundError } from '../errors.js';

type UserDelegate = Pick<PrismaClient['user'], 'create' | 'findMany' | 'findUnique' | 'update' | 'delete'>;

export class UsersService {
  constructor(private readonly users: UserDelegate) {}

  list(): Promise<User[]> {
    return this.users.findMany({ orderBy: { createdAt: 'desc' } });
  }

  async get(id: string): Promise<User> {
    const user = await this.users.findUnique({ where: { id } });
    if (!user) throw new NotFoundError('User not found');
    return user;
  }

  create(data: Prisma.UserCreateInput): Promise<User> {
    return this.users.create({ data });
  }

  async update(id: string, data: Prisma.UserUpdateInput): Promise<User> {
    await this.get(id);
    return this.users.update({ where: { id }, data });
  }

  async remove(id: string): Promise<void> {
    await this.get(id);
    await this.users.delete({ where: { id } });
  }
}
