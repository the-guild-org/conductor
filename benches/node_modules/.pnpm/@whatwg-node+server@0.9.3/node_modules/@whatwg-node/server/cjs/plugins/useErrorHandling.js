"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.useErrorHandling = exports.HTTPError = exports.createDefaultErrorHandler = void 0;
const fetch_1 = require("@whatwg-node/fetch");
function createDefaultErrorHandler(ResponseCtor = fetch_1.Response) {
    return function defaultErrorHandler(e) {
        return new ResponseCtor(typeof e.details === 'object'
            ? JSON.stringify(e.details)
            : e.stack || e.message || e.toString(), {
            status: e.statusCode || e.status || 500,
            headers: e.headers || {},
        });
    };
}
exports.createDefaultErrorHandler = createDefaultErrorHandler;
class HTTPError extends Error {
    constructor(status, message, headers = {}, details) {
        super(message);
        this.status = status;
        this.message = message;
        this.headers = headers;
        this.details = details;
        Error.captureStackTrace(this, HTTPError);
    }
}
exports.HTTPError = HTTPError;
function useErrorHandling(onError) {
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
exports.useErrorHandling = useErrorHandling;
