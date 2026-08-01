import { Client } from '@opensearch-project/opensearch';

export function createOpenSearchClient(node: string, username: string, password: string) {
  return new Client({ node, auth: { username, password } });
}
