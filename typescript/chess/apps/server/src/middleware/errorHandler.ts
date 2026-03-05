import { Hono } from 'hono';
import { DomainError } from '@chess/domain';

export const errorHandler = (c: any, next: () => Promise<void>) => {
    return next().catch((err: Error) => {
        if (err instanceof DomainError) {
            return c.json({ error: err.name, message: err.message }, 400);
        }
        console.error(err);
        return c.json({ error: 'InternalServerError', message: 'An unexpected error occurred' }, 500);
    });
};
