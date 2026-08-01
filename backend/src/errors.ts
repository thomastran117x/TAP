export class NotFoundError extends Error {}

export class ValidationError extends Error {
  constructor(public readonly details: unknown) {
    super('Request validation failed');
  }
}
