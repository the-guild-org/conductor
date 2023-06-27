import { PromiseOrValue } from '@envelop/core';
import type { GraphQLSchemaWithContext, YogaInitialContext } from '../types.cjs';
import type { Plugin } from './types.cjs';
export type YogaSchemaDefinition<TContext> = PromiseOrValue<GraphQLSchemaWithContext<TContext>> | ((context: TContext & YogaInitialContext) => PromiseOrValue<GraphQLSchemaWithContext<TContext>>);
export declare const useSchema: <TContext = {}>(schemaDef?: YogaSchemaDefinition<TContext> | undefined) => Plugin<YogaInitialContext & TContext>;
