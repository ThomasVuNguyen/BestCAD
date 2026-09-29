# BestCAD — web-first build plan

## Goal

Build a fully modifiable, browser-based parametric CAD application. The modeling workflow should feel familiar to an Onshape user, with command organization and a ribbon-style tool layout informed by Inventor. The model must be an exact, editable solid with feature history; a mesh displayed in the browser is only its visual representation.

The browser is the primary product surface. This lets Antigravity exercise the real UI and makes each change available as a repeatable CI run and a preview URL. A Mac desktop wrapper can come later if browser use proves insufficient. No dates are attached to the milestones; each ends with a demonstrable capability and a passing gate.

## Product contract

The first complete part workflow is a mounting bracket. A user can create a dimensioned sketch, extrude it, cut a hole, fillet an edge, change an earlier dimension, undo/redo, save/reopen, and export a valid STEP file. The browser shows a feature tree, an editable command panel, selection feedback, and precise failure messages. Every step must be possible through the browser UI and through the application's documented command API.

An enclosure is the second reference model for shell, patterns, and topology changes. A hinge is the later reference model for assemblies and drawings. Store their construction steps and expected geometric properties as test fixtures.

## Architecture

```text
Browser: React UI + Three.js viewport + local interaction state
                         │ typed commands, revisions, mesh + pick maps
                         ▼
Rust API: documents, transactions, regeneration, storage, jobs
                         │ narrow versioned geometry interface
                         ▼
Rust geometry worker + C++ OCCT: exact B-rep, tessellation, STEP
                         │
                         ▼
Versioned document records + rebuildable geometry/mesh artifacts
```

| Layer | Starting choice | Responsibility |
|---|---|---|
| Web UI | React, TypeScript, Vite | Feature tree, ribbon/commands, sketch editor, property panels, errors, undo/redo, document navigation. |
| Viewport | Three.js on WebGL2 | Orbit/pan/zoom, highlighting, face/edge/body picking, section display. Keep pick IDs tied to a particular mesh revision. |
| API | Rust with Axum | Validate and sequence commands, own document revisions, run regeneration jobs, expose import/export and stable browser APIs. |
| Geometry | Pin a tested OCCT release; initially evaluate 8.0.1 | Exact B-rep operations, shape validity, meshing, STEP import/export. Bridge Rust and C++ through a small, explicit interface such as `cxx`. |
| Sketch solver | Evaluate FreeCAD PlaneGCS in an isolated spike | Constraint solving and diagnostics. Begin with server-side authoritative solves; add browser-worker preview only if measured interaction latency requires it. |
| Persistence | In-memory state for M0; versioned feature documents in PostgreSQL from M1 | M0 proves geometry and browser interaction. M1 proves save/reopen and editable design history. STEP and mesh exports do not replace that history. |
| Deployment | One origin for built web assets and API, packaged with the geometry worker in a container | A single preview URL with no cross-origin setup. Add PostgreSQL to the local and preview stack in M1. |

Rust is useful for the service, command model, concurrency control, and deployment. OCCT's C++ algorithms still perform the expensive solid modeling; rewriting the kernel in Rust is outside this plan. Put OCCT behind a worker process before serving real users so a kernel crash cannot take down the document service. The first technical spike may use an in-process bridge to establish the API and build path.

### Decisions for the first slice

- **Geometry:** Build pinned OCCT 8.0.1 from source with CMake and connect it through `cxx` from M0. Cache the compiled dependency in local/container builds and CI. Do not build M0 on OpenCascade.js and then replace its kernel in M1; use WASM only for a measured need.
- **Database:** Defer PostgreSQL and migrations to M1. M0's box is deterministic, disposable in-memory state. The M0 one-command stack therefore starts the browser app and Rust/OCCT service without a database.
- **Preview:** Use the existing Coolify setup as the proposed deployment target. Connect a GitHub repository, configure pull-request previews with a dedicated preview domain and test-only settings, and run GitHub Actions for CI. Confirm server capacity, domain, and access during implementation; if any are missing, report the exact blocker while continuing local and CI work. A reachable preview URL remains the M0 exit gate.
- **Prior prototype:** The earlier Swift app, spike, and local reference checkouts were removed from this directory at the user's request. M0 starts from this plan and upstream project documentation; there is no Swift source to port or depend on.

### Model and API rules

- A document has stable IDs for features and sketch entities, typed parameters, dependencies, suppression state, units, and a schema version. Every edit is a validated transaction against an expected document revision.
- The API returns an exact revision, regeneration status, structured feature errors, tessellated mesh, and a mapping from current pick IDs to B-rep subshapes. Stale responses cannot overwrite a newer viewport.
- A transient triangle/face pick ID is distinct from a persistent design reference. Use OCCT operation history and semantic references where possible; surface an explicit repair path when topology makes a reference ambiguous.
- Keep the last valid model visible if regeneration fails. A failed or canceled command must leave document history consistent.
- Start with a simple single-user document model. Add auth before hosting real user files; add concurrent editing only after command and merge semantics have been proven.
- Publish an API/schema and a way to rebuild every shipped dependency. Track licenses, patches, and attribution for OCCT and any solver code.

## Browser and CI contract

These are requirements from the first vertical slice, because Antigravity must be able to test the application itself.

1. **One-command local run.** A documented command starts the web app, API, and geometry engine with a deterministic demo box in M0. Starting in M1, the same command starts PostgreSQL and seeds a demo document. No manual geometry commands are needed to reach the example model.
2. **Browser-accessible preview.** Every reviewable change has a full-stack preview URL with synthetic fixture data. It loads without access to private production documents. The UI exposes the same modeling controls in preview as in normal use.
3. **Automatable interface.** Use accessible button names and labels, stable selectors for canvas-dependent actions, deterministic demo documents, and a reset endpoint limited to test environments. The viewport must provide a repeatable way to target a known face/edge in end-to-end tests.
4. **UI evidence.** Playwright covers create → select → edit → regenerate → undo → save → reload → STEP export. Run Chromium and WebKit on the core flow. Save screenshots, trace, and console errors on failure. Antigravity reviews the preview URL and reports what it could actually perform in the browser.
5. **Geometry evidence.** For every feature operation, check B-rep validity, solid count, volume/bounds within tolerances, and STEP round-trip through an independent reader or process. A successful screenshot alone does not pass a modeling gate.
6. **CI gate.** Run TypeScript typechecking, lint, Rust format/lint/tests, geometry fixtures, production container build, and browser end-to-end tests; add database migrations in M1. Vite's TypeScript transpilation does not itself typecheck, so typechecking is explicit.
7. **Delivery gate.** Deploy the reviewed container only after the preview passes; include a database migration from M1 onward. Verify the deployed URL with a browser smoke flow, not just a healthy HTTP response.

## Milestones

### 0. Prove the stack in a browser

Set up the repository and one-command local stack without a database. Build a tiny page that asks the Rust API to create an OCCT box, receives its tessellation and face mapping, renders it, selects a face, changes a dimension, and downloads a STEP file. Build the same stack in GitHub Actions and expose a working Coolify pull-request preview URL. Measure cold build, geometry operation, mesh transfer, and browser interaction latency. Spike `cxx` and the geometry-worker boundary; record the chosen build and license strategy. Compare an OCCT WebAssembly proof using OpenCascade.js only if server latency or offline use becomes a measured issue.

**Gate:** Antigravity can open the preview URL and complete the box workflow in the browser. Playwright repeats it in CI. The generated B-rep and STEP pass geometric checks. Record measured results and the kernel integration decision.

### 1. Make a durable part document

Add PostgreSQL, migrations, and seeded preview documents. Implement versioned document schema, command transactions, save/reopen, regeneration status, undo/redo, feature tree, property editor, and basic primitive/extrude/cut/fillet operations. Add STEP import and export. Establish viewport selection of bodies, faces, and edges with revision-safe pick maps. Keep browser navigation and errors usable without developer tools.

**Gate:** Create a part, select a face, edit a dimension, undo/redo, close/reload, and export STEP entirely in the browser. Reopening regenerates the same valid solid from its feature history. Preview and CI exercise this path.

### 2. Complete the mounting bracket

Add datum-plane sketches, lines/rectangles/circles, geometric and dimensional constraints, solver feedback, profile validation, sketch extrude/cut, and dependency regeneration. Add sketch-on-face only after persistent reference behavior is demonstrated. Implement explicit repair for broken references.

**Gate:** Build the reference bracket through the UI. Edit its width and hole position; downstream features update or show the responsible repairable error. Undo, reopen, and export retain the intended geometry. Antigravity can run the whole scenario at a preview URL.

### 3. Make part modeling dependable

Prioritize revolve, chamfer, patterns, mirror, shell, sweep, loft, variables, units, multi-body operations, and rollback according to the bracket/enclosure fixtures. Refine the Inventor-inspired command groups and Onshape-inspired feature history based on real modeling sessions. Maintain regression fixtures for kernel failures and topology changes.

**Gate:** Both reference parts survive upstream edits and process restarts, remain valid B-reps, and export valid STEP. The UI preserves the last valid result and identifies failed features.

### 4. Assemblies and drawings

Define part instances, transforms, external/versioned references, and configuration references. Evaluate an open-source assembly solver against fixed, revolute, slider, and planar mates. Add assembly tree, interference checks, bill of materials, drawing views, dimensions, and PDF/DXF export.

**Gate:** Build and articulate the hinge, update one of its parts, reopen the assembly, and export a dimensioned drawing and bill of materials. Broken references appear as actionable errors.

### 5. Configurations, version history, and collaboration

Add named parameters and suppression variants, then immutable document revisions and branches. Define merges in terms of feature commands and references before adding simultaneous edits. Add sharing, permissions, comments, and live collaboration only with a tested conflict and recovery model.

**Gate:** Two bracket configurations share one editable feature history. Two branches can merge compatible edits or report a concrete conflict without silently altering a solid. A shared document can be reopened by an authorized user with the same geometry and history.

## First implementation task

Create the repo/CI skeleton and the milestone-0 vertical slice. The deliverable is a Coolify preview URL showing an editable OCCT box, with face selection and STEP export, backed by Rust and reproducible in GitHub Actions. Keep the slice deliberately small: its job is to prove the whole browser-to-kernel-to-preview loop before expanding the feature list.

The prior Swift/macOS prototype is not a runtime dependency for this plan. Its experiments informed the feature and kernel requirements, but the web UI and Rust document model should be built against the browser/API contracts above.

## Source references

- [OCCT releases](https://github.com/Open-Cascade-SAS/OCCT/releases)
- [Rust/C++ `cxx` bridge](https://cxx.rs/)
- [Axum documentation](https://docs.rs/axum/latest/axum/)
- [Three.js WebGL renderer](https://threejs.org/docs/pages/WebGLRenderer.html)
- [Vite TypeScript behavior](https://vite.dev/guide/features)
- [Playwright CI guidance](https://playwright.dev/docs/ci)
- [FreeCAD PlaneGCS source](https://github.com/FreeCAD/FreeCAD/tree/main/src/Mod/Sketcher/App/planegcs)
- [FreeCAD topological naming limitations](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/Topological_naming_problem.md)
- [OpenCascade.js WebAssembly option](https://github.com/donalffons/opencascade.js/)
