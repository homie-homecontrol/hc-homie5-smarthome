# hc-homie5-smarthome Tasks

Tracking file for follow-up work from the deep semantic architecture review.
Use this file to resume work in later sessions.

## Status Key

- [ ] pending
- [~] in progress
- [x] done

## Phase 1: High-priority semantic correctness

1. [x] Unify `SmarthomeType` canonical encoding
   - Problem: serde value and protocol `as_str()` value are inconsistent.
   - Scope:
     - `src/lib.rs` (`SmarthomeType` serialization/parsing paths)
     - Any call sites in sibling crates relying on current serde value
   - Goal: one canonical wire/config representation (prefer protocol-stable form).
   - Acceptance:
     - Roundtrip serde tests pass for all variants.
     - Existing matching helpers still work (`is_node_desc_type`, etc.).
   - Changelog:
     - Implemented manual `Serialize`/`Deserialize` for `SmarthomeType` in `src/lib.rs` using canonical constants from `as_str()`/`from_constant()`.
     - Added serde tests to verify canonical roundtrip for all variants and rejection of short names.

2. [x] Unify `SmarthomeAlert` canonical encoding
   - Problem: serde names and alert IDs diverge (e.g. `battery-low` vs `hc-battery-low`).
   - Scope: `src/alerts.rs`.
   - Goal: canonical encode/decode through one stable ID mapping.
   - Acceptance:
     - Tests cover every variant: serialize, deserialize, `as_str()`, `from_id()`.
   - Changelog:
     - Implemented manual `Serialize`/`Deserialize` for `SmarthomeAlert` in `src/alerts.rs` using canonical `hc-*` IDs via `as_str()`/`from_id()`.
     - Added serde tests verifying canonical roundtrip and rejection of short names.

3. [x] Fix thermostat `boost-time` gating bug
   - Problem: builder checks `config.boost_state` instead of `config.boost_time`.
   - Scope: `src/thermostat_node.rs`.
   - Acceptance:
     - Property appears only when `boost_time` exists.
     - Regression test added.
   - Changelog:
     - Fixed `boost-time` property gating in `src/thermostat_node.rs` to use `config.boost_time` instead of `config.boost_state`.
     - Added thermostat builder tests covering all config fields:
       - unit + temp_range mapping for `set-temperature`
       - each boolean gate (`valve`, `windowopen`, `boost_state`, `boost_time`, `mode`)
       - mode list mapping from `modes` to enum property format

## Phase 2: Parsing and diagnostics hardening

4. [x] Replace ambiguous parse return signatures
   - Problem: current `Option<Event>` hides parse errors vs no-match.
   - Goal: use explicit outcome type (e.g. `Result<Option<Event>, ParseError>`).
   - Scope:
     - node parsers in `src/*_node.rs`
     - consumer code in sibling crates using parser functions
   - Acceptance:
     - Invalid payload becomes observable and testable.
     - No behavior regressions for valid payloads.
   - Changelog:
     - Added public parsing API in `src/lib.rs`:
       - `ParseOutcome<T>` with `NoMatch`, `Parsed`, `Invalid(ParseError)`
       - `ParseError` + `ParseErrorKind` + `std::error::Error` implementation
       - `SetCommandParser` public trait
       - helper conversions: `ok()`, `into_result()`, and `From` impls
     - Replaced all set-command parser methods in node publishers (`switch`, `dimmer`, `shutter`, `colorlight`, `light_scene`, `thermostat`) with `parse_set` / `parse_set_event` returning `ParseOutcome`.
     - Removed old `match_parse*` APIs (breaking change, by design).
     - Updated consumers in `hc-hm2homie5` and `hc-deconz2homie5` to handle `ParseOutcome` explicitly and stop routing on `Invalid`.
     - Added `ParseOutcome` unit tests in `src/lib.rs` and validated with `cargo test`.

5. [x] Remove direct `println!` from library logic
   - Problem: noisy stdout side effects in library paths.
   - Scope: `src/light_scene_node.rs` and any similar spots.
   - Goal: use crate logger or remove debug output.
   - Acceptance:
     - No `println!`/`eprintln!` in non-test library code.
   - Changelog:
     - Removed parser debug `println!` calls from `src/light_scene_node.rs` while migrating to `SetCommandParser`.
     - Audited remaining `println!`/`eprintln!` usages; current occurrences are in `src/lib.rs` integration tests (`#[cfg(test)]`) only.

## Phase 3: Config compatibility and semantic consistency

6. [x] Add `#[serde(default)]` strategy to config structs
   - Problem: future fields can break backward compatibility.
   - Scope: config-like structs across node modules.
   - Acceptance:
     - Adding optional fields is backward compatible.
     - Tests verify old minimal config samples still deserialize.
   - Changelog:
     - Added `#[serde(default)]` to all node config structs in `hc-homie5-smarthome` (`Switch`, `Dimmer`, `Shutter`, `Colorlight`, `LightScene`, `Thermostat`, `Weather`, `Motion`, `Vibration`, `Maintenance`, `Button`, `Powermeter`).
     - Added serde default tests in `src/lib.rs` that verify:
       - `{}` deserializes to `Default` for all node config types.
       - partial configs preserve defaults for missing fields (thermostat and light scene examples).

7. [ ] Audit and fix property metadata consistency
   - Problem: some names/units/descriptions appear copy-paste incorrect.
   - Scope: all node builder property definitions.
   - Acceptance:
     - Metadata table reviewed for each property.
     - Incorrect labels/units corrected with tests or snapshots.

8. [ ] Normalize ID definition style
   - Problem: mixed usage of `&str + try_into().unwrap()` and const `HomieID` patterns.
   - Scope: all `*_node.rs` modules.
   - Goal: consistent, safer ID initialization style.
   - Acceptance:
     - No new `unwrap()` introduced for static ID conversion paths.

## Phase 4: Architecture and maintainability improvements

9. [ ] Split semantic layer from transport adapter
   - Goal: decouple domain semantics from direct MQTT `Publish` construction.
   - Approach:
     - Introduce semantic event/intent model.
     - Keep Homie5 publish mapping in adapter layer.
   - Acceptance:
     - Domain behavior testable without MQTT publish objects.

10. [ ] Reduce builder/publisher boilerplate
    - Goal: remove copy-paste risk across node modules.
    - Options:
      - internal macro/template
      - shared helper traits/builders
    - Acceptance:
      - At least 2 node modules migrated to prove pattern.
      - No loss of readability.

## Testing program (cross-cutting)

11. [ ] Add table-driven semantic tests per node
    - Coverage targets:
      - node description schema (datatype/unit/retained/settable)
      - set command parsing (valid + invalid)
      - state-to-publish generation
      - serde roundtrip compatibility
    - Acceptance:
      - Significant increase from current baseline test coverage.

## Notes for future sessions

- Start with items 1-3 before doing broad refactors.
- Keep changes backward compatible unless explicitly approved.
- After each completed item, update status and add a short changelog note under the task.
