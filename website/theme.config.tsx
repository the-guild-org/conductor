/* eslint sort-keys: error */
import { defineConfig, PRODUCTS } from '@theguild/components';

// bump 1
export default defineConfig({
  docsRepositoryBase: 'https://github.com/the-guild-org/conductor/tree/master/website', // base URL for the docs repository
  websiteName: 'Conductor',
  logo: PRODUCTS.CONDUCTOR.logo({ className: 'w-9' }),
  description: 'All-in-one GraphQL Gateway',
});
