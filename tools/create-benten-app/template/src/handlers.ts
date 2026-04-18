// Sample handler for the {{name}} project.
//
// `crud('post')` registers create / get / list / update / delete handlers
// with sensible defaults: no capability required, local storage, schema
// inferred from first write. See docs/QUICKSTART.md for the full zero-
// config path, and docs/DSL-SPECIFICATION.md for the options shape that
// adds capability checks, schemas, and custom transforms.

import { crud } from "@benten/engine";

/**
 * Zero-config CRUD handler set for the `post` content type.
 *
 * The returned value is a subgraph bundle ready to pass to
 * `engine.registerSubgraph(...)`. It exposes `.id`, `.actions`, and
 * (once registered) `.toMermaid()` for visual inspection.
 */
export const postHandlers = crud("post");
