# Source inventory schema and runtime review

Read-only source audit, 2026-11-11. The recovered inventory contract separates source instance IDs from config IDs, retains the four server collections, and takes capacity from `SItems` / `SBagSizeChange`. No authoritative default backpack capacity, fixed quick-use slot count, or starter contents were recovered. Fixture `ItemKind` values must not be used as source IDs.

The audit used bounded reads of `export/BagMgr.cs`, `export/gs.bag.scmsg`, `export/cfg/ItemCfg.cs`, `export/Share/Octets.cs`, and selected unpatched native methods from the paired APK. It used the existing Rust/Capstone helper without importing a native REA target or executing the game. No Python, GUI, Android runtime, service changes, or Waydroid retry was used.

## Evidence and limits

- APK: `export/lite4.0.apk`, SHA-256 `2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.
- Native ELF: `/tmp/codex-minimap-review/libil2cpp.so`, 76,608,224 bytes, SHA-256 `2e2efde1a103549694b17b361cacc18faa5714d06c444945e44b30f1377e2b3c`.
- APK/export pairing: [minimap-dump-pairing.md](minimap-dump-pairing.md).
- Main handler byte ranges, raw bytes, disassemblies and hashes: [inventory-native/manifest.json](inventory-native/manifest.json).
- Ancillary handler ranges and disassembly hashes: [inventory-native/ancillary-manifest.json](inventory-native/ancillary-manifest.json).
- Independent decoder cross-check ranges: [inventory-native/decoder-manifest.json](inventory-native/decoder-manifest.json).
- Native `getType()` IDs and exact return paths: [inventory-native-gettypes/README.md](inventory-native-gettypes/README.md).

All selected addresses were checked against the ELF program headers: the ranges lie inside a `PT_LOAD` with `p_offset = p_vaddr = 0`, so virtual addresses equal file offsets. The load base was not observed. Decompiled pointer arithmetic and unresolved-call placeholders are not treated as reliable behavior by themselves; the rules below use exported field layouts plus native branches/stores where needed.

Each recovered base method can be replaced by IFix. The original runtime IFix state, original network frame/session envelope, live server snapshots, server starter-generation logic, and runtime quick-dictionary enumeration order remain unobserved. Native claims below are therefore static base-method observations.

## Identity and plain item record

`gs.bag.scmsg.BagItem` has no `uniqueID` field. Its signed `instanceId` identifies an item instance; its signed `itemId` is the value passed to `ItemCfg.Get`, which keys `cfg.ItemCfg.id`. The separate config field also named `ItemCfg.itemId` is not interchangeable with either of these identities. Source declarations are in `export/gs.bag.scmsg/BagItem.cs` lines 13–41.

| Wire order | Field | Type |
|---:|---|---|
| 1 | `instanceId` | signed little-endian i32 |
| 2 | `itemId` | signed little-endian i32 |
| 3 | `number` | signed little-endian i32 |
| 4 | `duration` | signed little-endian i32 |
| 5 | `timeout` | signed little-endian i32 |
| 6 | `isBind` | one-byte bool |
| 7 | `bindTimeout` | signed little-endian i32 |
| 8 | `extraInfo` | `Octets`: signed i32 byte count, then opaque bytes |

An item with empty `extraInfo` occupies 29 bytes. `extraInfo` remains opaque; the general bag DTO does not identify a polymorphic payload type. No instance/config ID translation should be inferred from fixtures.

## Message bodies and protocol IDs

The default unmarshal layouts are listed below. A code XORs **each byte** before interpreting the resulting scalar. Collection counts, quick-map keys, and nested item records use the collection's code. `Octets` length and payload use their field's code. Uniform-code unmarshal overloads apply the supplied code throughout. Signed scalar values must be retained until an explicit domain admission policy is applied.

| Message | Base native type ID | Ordered fields and default XOR codes |
|---|---:|---|
| `SItems` | 59771964 | bag list `123`, skin list `14`, quick map `<i32,BagItem>` `7`, equip list `63`, showEquip bool `81`, capacity i32 `106` |
| `SItemChanged` | 59771867 | bindTimeout `99`, isBind `109`, timeout `100`, itemId `115`, duration `82`, instanceId `0`, extraInfo `4`, num `109` |
| `SQuickUseItemChanged` | 59771872 | timeout `46`, index `118`, isBind `97`, instanceId `49`, bindTimeout `47`, duration `126`, num `67`, itemId `7`, extraInfo `104` |
| `SBagSizeChange` | 59771878 | size i32 `77` |
| `SItemDurationChanged` | 59771842 | duration i32 `101`, instanceId i32 `88` |
| `SItemNumberChange` | 59771840 | isBind bool `36`, change i32 `57`, itemId i32 `123` |
| `SSetQuickUseItem` | 59771914 | index i32 `47`, instanceId i32 `123` |
| `SPutToHand` | 59771892 | instanceId i32 `127` |
| `SSyncBagItem` | 59771844 | one nested BagItem with uniform code `62` |
| `SClearAllBag` | 59771886 | empty body |

These IDs are native `getType()` return values, not metadata tokens or IFix patch IDs. Decoding an explicit message body does not reconstruct its surrounding frame, session encryption, or message timing. `SItems` masks are corroborated in `export/gs.bag.scmsg/SItems.cs` at lines 921/952, 1018/1047, 1105/1119/1135, 1165/1190, and 1245/1247.

An independent review of the implemented decoder reread all ten default body masks from the exported source. It also checked native `SItemChanged.unmarshal` (`0x10D7FE4–0x10D80E7`) and `SQuickUseItemChanged.unmarshal` (`0x10E1988–0x10E1A9F`) for each code and destination field offset, with no mismatch found. `Octets.pop_int(code)` at `0x21AD2E4–0x21AD34B` XORs each of the four bytes before placing them at bit offsets 0/8/16/24. Coded byte-array pop at `0x21ADCA4–0x21ADD13` XORs every copied byte in place. The retained slices corroborate the decoder's scalar endianness and opaque-payload XOR behavior.

The reconstruction decoder is [crates/game_data/src/inventory_wire.rs](../../crates/game_data/src/inventory_wire.rs). Its body-size, collection-size, and opaque-payload limits, rejection of duplicate quick keys, canonical bool requirement, and malformed-data checks are reconstruction policy; they are not recovered game capacity limits. Its DTOs preserve signed values.

## Snapshot, capacity and sort

`RealBagCapacity` returns `_bagCapacity`. `BagCapacity` returns `max(_bagCapacity, bag entry count)`. The native getter ranges are `0x1942BD0–0x1942C73` and `0x1942C74–0x1942CBB`. `SBagSizeChangeHandle` at `0x194BC78–0x194BD3B` overwrites stored capacity and does not truncate inventory.

`SItemsHandle` at `0x1953EB0–0x19541DF` replaces bag, skin, quick-use, and equipment collections with the incoming collections, accepts the incoming capacity, sorts the bag, and resets the held instance to `-1`. The wire DTO also carries `showEquip`, but this base handler does not directly read or assign that field; preserving it at the authority boundary does not establish a BagMgr visibility setter. Sparse server quick keys carry the source slot/index identity; neither their highest value nor the count proves a fixed maximum slot count.

The bag comparison at `0x1963988–0x1963A3F` sorts ascending by `ItemCfg.order`, then `BagItem.itemId`, then `BagItem.instanceId`. The config lookup is keyed by the item's source `itemId`, not its instance ID.

Snapshot initialization at `0x1954DD4–0x19551AB` adds `ceil(UnityEngine.Time.realtimeSinceStartup)` to bag and quick timeout only when the item config's `existTime > 0`. The clock wrapper at `0x3423970–0x3423997` resolves the exact string at `0x3707122`: `UnityEngine.Time::get_realtimeSinceStartup()`. Skin/equipment timeout normalization was not observed in this initialization path.

Wire DTOs should retain raw timeout values. Reconstructing local timeout normalization requires an explicit receive-time `realtimeSinceStartup` value; it must not substitute wall-clock epoch time or a guessed time-to-live rule.

## Changed and quick-changed items

`SItemChangedHandle` (`0x1952174–0x19530CB`) and `SQuickUseItemChangedHandle` (`0x194867C–0x1948D8F`) use lookup `0x1948D90–0x1948FDF`: search bag first, then quick only. The lookup does not search skin or equipment.

| Case | Observed collection behavior |
|---|---|
| Known instance, nonzero `num` | Overwrite `number`, `extraInfo`, `isBind`, `bindTimeout`, and `duration`; preserve existing `instanceId`, `itemId`, and `timeout`. |
| Unknown instance, nonzero `num` | Create a complete record from the message. Add `ceil(realtimeSinceStartup)` to its timeout unconditionally. |
| Unknown in bag/quick, zero `num` | Do not create a record. If any equipment item has `itemId == msg.itemId`, remove the first equipment item with `instanceId == msg.instanceId`; otherwise leave collections unchanged. |
| Known instance, zero `num` | Remove the first matching bag entry, otherwise the first matching quick key. Do not remove every alias or a skin/equipment entry. |
| Quick change, known bag, nonzero `num` | Remove that object from bag and assign it to `quick[msg.index]`. |
| Quick change, known quick, nonzero `num` | Update that object at its existing quick key; do not relocate it to `msg.index`. |

For known quick-change zero removal, `0x1948858` branches to `0x19489DC`, and `0x1948A04` calls the removal helper. It subsequently updates binding/duration and fires events using the detached object, but never reaches the quick dictionary assignment at `0x19488F0`. It does not reinsert a zero-count record. The collection removal helper is `0x1949324–0x19495C3`.

The equipment fallback occurs only when the active bag/quick lookup found nothing and `num == 0`. ItemChanged follows `0x1952548 → 0x1952728`, checks equipment config-ID membership, compares equipment instance IDs at `0x1952794–0x19527A0`, and removes one entry at `0x1952EA0`. QuickChanged follows `0x19488F8 → 0x1948B60`, with equivalent membership/search logic and removal at `0x1948CAC`. The retained `get_EquipItemsIds` helper (`0x19430D8–0x19432BF`) constructs a fresh set from current equipment entries' `itemId` fields; the gate is not a static equipment-type allowlist. Neither fallback searches or removes skin items.

Both handlers branch on `num == 0`, not `num <= 0`. Positive-number validation is a sensible reconstruction admission policy, but not a recovered native source restriction. Unique instance IDs within quick, deterministic alias handling, and positive/config-known ID validation likewise require an explicit policy if the reconstruction cannot retain the source dictionary's runtime enumeration order. Such policy must not be described as an observed source invariant.

The source handlers also maintain derived gun/water/other domain data and trigger UI events. The above table specifies recovered inventory collection state; it does not reconstruct all item effects or derived caches.

## Duration and number notifications

`SItemDurationChangedHandle` (`0x1951EF0–0x1952173`) uses the all-container lookup (`0x194738C–0x194769B`): bag, then quick, then skin, then equipment. It changes only `duration` on the first matching instance. An unknown instance causes no inventory mutation.

`SItemNumberChange` is an alert/notification by config ID, binding state, and count change. Its handler does not mutate the bag or a particular instance. Applying `change` as an inventory delta would infer behavior absent from the recovered handler.

## Ancillary message semantics

### `SClearAllBag`

The empty body does not mean emptying every collection. `SClearAllBagHandle` (`0x194BA50–0x194BB3B`) calls `Clear()` (`0x194BB3C–0x194BC1F`). The latter's complete base path calls `ClearNoBindGunDate`, `ClearList(bag)`, `ClearDic(quick)`, `ClearList(skin)`, and `ClearList(equip)`, then clears equipment-skin IDs and bag-bottle-water data and nulls thermal-sight extra data. The clear path does not write stored capacity, held instance, or `showEquip`.

`ClearList` (`0x19624FC–0x19625FB`) removes entries with `BagItem.isBind == false` and preserves bound entries. `ClearDic` (`0x19625FC–0x19628EB`) gets `ItemCfg` using the quick item's `itemId`, then checks:

```text
0x19627C4: load config byte at 0x2C
0x19627C8: if nonzero, continue enumeration
0x19627CC: load BagItem byte at 0x24 (isBind)
0x19627D0: if nonzero, continue enumeration
otherwise enqueue this quick key for removal
```

Export `cfg.ItemCfg.cs` lines 44–45 identifies config offset `0x2C` as `isCanDiscard`; `BagItem` offset `0x24` is `isBind`. Thus the observed base condition preserves quick entries when **`ItemCfg.isCanDiscard || BagItem.isBind`**, and removes only entries where both are false. The condition's name is counterintuitive, but its native polarity must not be reversed to make it sound intuitive. A null config goes to the native failure path rather than behaving as a default false flag.

The handler and both filtering helpers are retained in the ancillary manifest. Reconstructing this message may apply the proven collection filters, but does not establish downstream gun/cache behavior without additional work.

### `SPutToHand`

The unpatched base handler stores the incoming instance ID directly: `0x194C7A8` loads the signed message field at `0x18`, and `0x194C7B8` stores it at BagMgr `0xA0`, `_handInstanceId` (`export/BagMgr.cs` lines 5822–5824). The store precedes any inventory lookup or positive-ID branch. It does not relocate or remove an item from any inventory collection. Unknown IDs and negative values are not rejected by this store.

After storing, it fires events, branches on whether the held ID is at least one, invokes `SmartPutAllGunPart`, queries a player weapon, or hides all weapons. Those weapon/visual effects remain outside the reconstructed inventory state. The retained range `0x194C710–0x194C92B` ends before the `SmartPutAllGunPart` helper.

### `SSetQuickUseItem`

The body and native protocol ID are recovered, but its runtime contract is more than assigning `quick[index]`. The native base handler at `0x1950C68` distinguishes negative indices, bag versus quick locations, existing destination objects, quick swaps, bag moves and coffer bookkeeping; it also updates held-weapon behavior, sorts the bag and emits alerts/events. The retained `0x1950C68–0x19513CF` slice includes its `GetQuickUseItemByIndex` helper at `0x1951304`.

The complete domain behavior was not reconstructed in this pass. Keep application of this message unsupported until all relevant branches/helpers are modeled and verified. A simple direct setter would silently discard source behavior.

### `SSyncBagItem`

Its body contains one `BagItem` with code `62`, and its native type ID is established. No BagMgr handler or registration was found in the bounded source review. This absence is not proof that the game never handles the message elsewhere. Keep its runtime application unsupported until a handler is established; do not infer snapshot/upsert semantics from the name.

## Stack limits and authoritative defaults

`ItemCfg.maxPile` is not the only runtime pile limit. `OpenServerMgr.GetMaxPileNum(defaultValue)` (`0x1800B20–0x1800D13`) can return an active custom-community value instead of its argument; `export/cfg/CommunityType.cs` line 34 identifies `MaxPileNum = 9`. Existing `SourceInventory.stack_limit` therefore does not prove the maximum allowed count in an authoritative snapshot. Rejecting all snapshots above that static config value would conflict with the observed override mechanism.

The BagMgr constructor, capacity getters, snapshots and capacity-change handlers did not establish an authoritative starter loadout, default backpack capacity, or fixed quick-use capacity. An empty disconnected reconstruction must describe those values as unknown, and populate source state only from explicit validated authority data. Simulated fixture grants belong to the fixture domain and must not synthesize source items.
