import { Response as DefaultResponseCtor } from '@whatwg-node/fetch';
export function createDefaultErrorHandler(ResponseCtor = DefaultResponseCtor) {
    return function defaultErrorHandler(e) {
        return new ResponseCtor(typeof e.details === 'object'
            ? JSON.stringify(e.details)
            : e.stack || e.message || e.toString(), {
            status: e.statusCode || e.status || 500,
            headers: e.headers || {},
        });
    };
}
export class HTTPError extends Error {
    constructor(status, message, headers = {}, details) {
        super(message);
        this.status = status;
        this.message = message;
        this.headers = headers;
        this.details = details;
        Error.captureStackTrace(this, HTTPError);
    }
}
export function useErrorHandling(onError) {
    return {
        onRequest({ requestHandler, setRequestHandler, fetchAPI }) {
            const errorHandler = onError || createDefaultErrorHandler(fetchAPI.Response);
            setRequestHandler(async function handlerWithErrorHandling(request, serverContext) {
                try {
                    const response = await requestHandler(request, serverContext);
                    return response;
                }
                catch (e) {
                    const response = await errorHandler(e, request, serverContext);
                    return response;
                }
            });
        },
    };
}
