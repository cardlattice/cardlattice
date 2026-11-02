import { defineConfig } from 'orval';

export default defineConfig({
  catalogue: {
    input: '../../docs/openapi.json',
    output: { target: 'src/generated/catalogue.ts', client: 'react-query', httpClient: 'fetch', override: { mutator: { path: './src/api.ts', name: 'request' }, query: { useQuery: true }, fetch: { includeHttpResponseReturnType: false } } },
  },
});
