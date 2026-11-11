# Stand/Crouch invocation and transition review

2026-11-11. Bounded static review of the supplied recovered APK/native dump.
These files describe unpatched base procedures; they do not observe original
game execution, IFix state, committed transitions or REA provider Evidence IDs.
The existing Rust/Capstone helper was reused unchanged under the previously
authorized read-only fallback. No Python, Cargo build, GUI, service, cache
recovery, cleanup or system mutation was performed. Only this evidence folder
was written.

The exact APK, ELF, helper/source, C# and artifact SHA-256 identities and native
ranges are retained in [manifest.json](manifest.json). Every retained range
decoded fully; each asm line contains the address, native four-byte word and
decoded instruction. Bounds marked `slice` establish only the stated local
claim, while ordinary entries retain the complete body through its final
return/error path. Reproduce a range with:

```text
/tmp/codex-minimap-review/native disasm /tmp/codex-minimap-review/libil2cpp.so RVA LENGTH
/tmp/codex-minimap-review/native words /tmp/codex-minimap-review/libil2cpp.so RVA LENGTH
```

## Complete OnClickCrouch2 caller contract

The declaration is `SC.UI.BattlePanel.OnClickCrouch2`,
`export/SC.UI/BattlePanel.cs:81777`, token `0x6010B45`, RVA `0x14d53c8`.
The full native body ends before `0x14d56f8` (length `0x330`), retained in
[ui-click-crouch2.asm](ui-click-crouch2.asm). IFix 3311 can replace the whole
procedure; an active patch bypasses all ordinary branches.

`BattlePanel.m_Self` is the panel player at instance offset `0x28`
(token `0x4008CE4`). Player `FSM` is at `0x500` and `FSMUpBody` at
`0x508` (tokens `0x4001BFC` / `0x4001BFD`). Both current-state ID
reads use the same generic `BaseFsmState<T>.get_ID` method/descriptor,
callee `0x2dd4cb4` and descriptor slot `0x46000880`. The recovered
`StateID` declarations retain `Attack = 16`, `Pa = 2`, `PaUp = 3`,
`Stand = 1`, `Crouch = 4`, `CrouchDown = 5`. The two FSM channels
remain distinct; an upper-body Attack gate does not mean a base-state Attack
gate. Do not adopt damaged external-type field offsets as a Rust layout.

On the ordinary non-null path, the caller does the following, in order:

1. Read `m_Self.FSMUpBody.CurrentState.ID`. If it is Attack (16), return
   immediately. Missing panel player/FSM/current state throws through the
   native null-error path; it is not an ordinary no-op.
2. Read `m_Self.FSM.CurrentState.ID`. If it is Pa (2), call
   `m_Self.CheckCanPaUp()` at `0x14d54f8`, callee `0x1ec332c`.
   A true result leads to `m_Self.FSM.SwitchState(PaUp, null)` and a tail
   return. No audio is requested by this branch. A false result falls through.
3. Read `m_Self.FSM.CurrentState.ID` again. If it is Stand (1), obtain the
   audio singleton and call `AudioManager.Play2D(339)` at `0x14d55a4`.
   The integer return is ignored. Then resolve `Battle.Ins.SelfPlayer.FSM`
   and tail-call `SwitchState(Crouch, null)`.
4. Otherwise read `m_Self.FSM.CurrentState.ID` again. If it is Crouch (4),
   call `m_Self.CheckCanCrouchUp()` at `0x14d5668`, callee `0x1eb3344`.
   False returns without a switch/audio call. True calls
   `m_Self.FSM.SwitchState(Stand, null)` at `0x14d56cc`; after that
   call returns, obtain the audio singleton and call `Play2D(435)`
   at `0x14d56f0` via a tail branch.
5. Other IDs return without an action.

The initial Pa ID read and the later Stand/Crouch reads are separate source
observations. `CheckCanPaUp` can have runtime effects; a reconstruction
continuation must allow the host to supply the later ID reads instead of
assuming they match the first one. The callback requests use generic
`BaseFsmSystem<T>.SwitchState`, callee `0x2dd568c`, with null args.
No `m_CanExit`, `CanBreak`, ActorStatus, input magnitude, Rush gate or
switch-return verdict is checked in this caller. A returned switch call does
not prove the requested state was committed; global switch arbitration and
callbacks remain external.

The audio IDs 339/435 are **Play2D IDs**, not event IDs. Declaration:
`export/AudioManager.cs:2881`, token `0x6003AC5`, RVA `0x20e2a40`.
No playback parameters, asset identity or full audio behavior are inferred.
Stand audio occurs before the global player/FSM lookup and switch; Crouch
audio occurs after the panel FSM switch returns. If audio lookup/call fails,
the later action is not reached. If the Crouch switch throws, audio 435 is
not reached. The caller tests no successful-commit flag before requesting it.

## Global player owner is authenticated

The Stand branch's target is `Battle.Ins.SelfPlayer.FSM`; it is not
implicitly the same object as `BattlePanel.m_Self.FSM`.

`export/Battle.cs` retains `Ins` as a static field at `0x88`
(token `0x4000408`, line 9343), `SelfPlayer` as an instance field at
`0x20` (token `0x40003C3`, line 9067), and `LoadFinish` at
`0x18` (token `0x40003C1`, line 9059). The focused
[battle-awake prologue](battle-awake-prologue-slice.asm) establishes
`x19 = this` in `Battle.Awake` (declaration RVA `0x20503f4`).
The [store slice](battle-awake-ins-store-slice.asm) loads the same TypeInfo
slot `0x45e04a0` used by the click procedure, obtains its static fields
and stores `x19` to offset `0x88` at `0x20508f0`. The following
instruction clears this instance's `LoadFinish` byte at `0x18`.
These slices authenticate the owner/field chain; they do not reconstruct
the rest of Awake or prove runtime equality of the two player references.

## Move invocation and update order

Complete [stand-act.asm](stand-act.asm) (`StandState.Act`,
`0x1e7b5f4..0x1e7b748`) and [crouch-act.asm](crouch-act.asm)
(`CrouchState.Act`, `0x1e618c8..0x1e6197c`) show that after their
IFix bypasses, Move is unconditional on the admitted non-null path:

- Stand sets animator Horizontal from `BasePlayerController.InputVector.x`
  with delay 0.12 and Forward from its Y with delay 0.1; it requests the
  sex-dependent base animation state, then calls Move at `0x1e7b6f4`.
  After Move it evaluates the bare-hands speed-up/reset path.
- Crouch calls Move first at `0x1e61914`, then sets Horizontal and Forward
  from `InputVector`, both with delay 0.12.
- The native f32 constants are retained separately: `0x361f9e8 =
  0x3df5c28f` (0.12) and `0x361f580 = 0x3dcccccd` (0.1).
- Neither Act procedure gates Move on `_canNoRootMotion`, magnitude,
  CannotMove, CanRush, grounded state or elapsed posture delay. Downstream
  velocity and FixedUpdate rules remain separate.

Complete [player-update.asm](player-update.asm) (`PlayerController.Update`,
`0x1eb568c..0x1eb5930`) decrements selected timers before returning for
`IsDie` (instance `0x120`) or false `Battle.Ins.LoadFinish`.
On the ordinary admitted path it updates Rush/selected controller fields and
input/IK helpers, calls base `FSM.Update` at `0x1eb58c4`, then upper
`FSMUpBody.Update` at `0x1eb58d4`, both callee `0x2dd5d58`.

The complete [generic FSM update](fsm-update.asm)
(`0x2dd5d58..0x2dd5e20`) returns when `CanRun` (instance `0x28`)
is false. Otherwise it invokes `Ticker` through `Utils.TriggerEvent`,
re-reads CurrentState and calls its virtual Act (slot 8), then re-reads
CurrentState again and calls Reason (slot 9). State changes made by Ticker
or Act therefore affect the later invocation. This is an invocation
contract, not physical movement or state-commit parity.

## Entry and posture coroutine boundaries

The complete Stand/Crouch entry and coroutine bodies are retained for the
following limited claims:

- Stand entry calls `StandCapsule` (`0x1e7b318`), sets
  `_canNoRootMotion=false` (`0x68`) and clears delay (`0x6c`).
  An exactly two-entry args array supplies a boxed f32 delay from args[1].
  Exactly one argument enters a separate vector-position branch. Other
  admitted lengths preserve the cleared delay. Entry includes other
  camera/collider/weapon/state callbacks; those have not been ported here.
- Crouch entry calls `CrouchCapsule` (`0x1e6151c`), then its skill
  speed helper, sets `_canNoRootMotion=false` (`0x50`) and delay=0
  (`0x54`). Nonempty args supplies a boxed f32 from args[0]; empty args
  keeps zero. A null/wrongly typed source args object is exceptional.
- Each ActCoroutine waits through `Utils.WaitForSeconds` using its selected
  current delay and then sets `_canNoRootMotion=true`. Stand's coroutine
  does not set Animator.applyRootMotion. Crouch's does set
  `PlayerAnimator.applyRootMotion=false` after setting the flag.
- Entry changes the capsule before the coroutine delay completes.
  Do not substitute the delay as a Move invocation gate or make the Stand
  coroutine perform Crouch's Animator setter.

Complete [stand-reason.asm](stand-reason.asm) sets NeedHeadIK true and tail-calls
CheckGroudDis. The unpatched [crouch-reason.asm](crouch-reason.asm) is a no-op.
The ground-distance procedure is not reconstructed in this pass.

## Distinct CrouchDown and legacy click paths

The complete CrouchDown entry/change-animation/leave/Act/Reason bodies are
retained. Entry clears `m_CanExit` (`0x50`), disables the upper FSM,
switches its state to numeric 0, requests the weapon-dependent target
animation and stops a current gun's shooting. In the unpatched procedure,
entry args are not consumed after the patch check. Act closes IK and calls
ChangeVelocity(0). Leave enables the upper FSM.

CrouchDown.Reason's complete gate is `Player.IsPlayFinish(target-animation)`
(callee `0x1e52520`). If false, return. If true, request base
`SwitchState(Pa, null)`, call EnableUpBodyFsm, and if CurGun is null
request upper `SwitchState(NullStateID, null)` (numeric 0).
`m_CanExit` is not read by the complete ordinary Act/Reason bodies.
No timer/field-based exit gate should be invented from its declaration.

The separate legacy `SC.UI.BattlePanel.OnClickCrouch`
(`0x14ddd9c..0x14ddf50`, [ui-click-crouch.asm](ui-click-crouch.asm))
only acts when the panel player's base state is Crouch: it requests
CrouchDown on `Battle.Ins.SelfPlayer.FSM` with a one-element object
array containing the exact f32 0.5. This argument is not consumed by the
ordinary CrouchDown entry body reviewed above. The separate
`OnClickStand` (`0x14ddf50..0x14de0f4`,
[ui-click-stand.asm](ui-click-stand.asm)) only acts when the panel
base state is Stand: Play2D(339), then global base SwitchState(Crouch, null).
These names do not justify merging the handlers into a generic posture toggle.

## Remaining boundaries

Implement a source-only explicit continuation for OnClickCrouch2 that preserves
fresh ID reads, explicit clearance verdicts, panel/global switch targets and
audio-before/after call order. Require the host to acknowledge calls returning;
do not synthesize state commits, source readiness or authentic input bindings.
CheckCanPaUp must use its own externally supplied verdict until its complete
query/aggregation and effective runtime inputs are admitted. It is not the
PaToStand cast.

Effective transforms/capsule scaling, source IFix overrides, switch arbitration,
root motion/animation displacement, jump/landing and physical speed/gravity
remain prerequisites for active original movement. The fixture controller and
desktop C binding are not validated by this static audit.

