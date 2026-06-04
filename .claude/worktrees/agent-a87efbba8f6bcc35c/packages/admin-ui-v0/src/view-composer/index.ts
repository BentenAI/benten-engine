// G24-C wave-6b — view-composer module entry-point.
//
// Re-exports the `ComposedViewCreator` surface + the
// `UserViewSpec` shape consumers (browser-tab + Tauri webview)
// import.

export { ComposedViewCreator } from "./composed_view_creator.js";
export type {
  ComposedViewCreatorBridge,
  PreviewState,
  SaveOutcome,
  SubscribeCursor,
} from "./composed_view_creator.js";
export type {
  LabelPattern,
  TypedOutputProjection,
  UserViewSpec,
} from "./view_spec.js";
export { userViewSpec } from "./view_spec.js";
