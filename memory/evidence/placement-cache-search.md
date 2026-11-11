# Bounded local placement-cache search — 2026-11-11

A filename-first, read-only search found no authentic `sceneinfo/*.oc`
or `objectInfo.obj` filenames in these task-authorized roots:

- Project `assets/`.
- Project `export/`.
- Project `.local/`, including `.local/apk-runtime-candidates/`.
- `/tmp/codex-minimap-review/`.
- `/tmp/codex-memory-dump-review/`.

The extractor confirmed no other task-owned source-cache directories under
`/tmp`. The prior bounded APK/FileList absence scan was not repeated.
Known `.oc` files in the searched roots are gameplay configuration and the
retained `/tmp/codex-minimap-review/mapcfg.oc` topology input; their extension
does not authenticate a SceneInfo placement body.

The one placement-like local candidate is `assets/imported_map/placements.bin`:

- Size: 2,561,901 bytes.
- SHA-256:
  `8207bde27ca1e529e6b564fb3731d5427d1f03b5883b5ef99c6bdd78f765be50`.
- First eight bytes: `MSPLC002`.
- The retained [map evidence](../map-evidence.md) attributes it to separate
  XAPK/legacy bucket processing. It is not an authentic recovered SceneInfo
  payload and must not be passed to the strict source decoder as one.

The result covers these local filenames only. It does not exclude arbitrary
renamed, packed or encrypted payloads, uninspected downloaded content, or
runtime/server-provided scene data. No unrelated home directories, private
session history, system caches or original Android runtime were searched.
No files were deleted or rewritten by the search.

The next source-placement milestone therefore still requires an authentic
payload with its exact carrier/path and digest, followed by a complete
decode and source-backed prefab placement. Synthetic reader fixtures and
the legacy binary cannot close that evidence gap.

## Later bounded Waydroid extension

At the user's request, the audit was extended to the exact package
`com.herogame.gplay.survival.lite`, identified in the retained
[Android runtime check](android-runtime-check.json). The project source APK
and installed counterpart are distinct files; the prior comparison established
that their asset members match while native/metadata members differ.

Known package directories exist under
`/home/andlua/.local/share/waydroid/data/data/` and `user_de/0/`, with mode
700 and owner `nobody:nobody`. Filename enumeration of those two exact
package directories and the package-scoped external `Android/data` and
`Android/obb` directories was denied. The five exact standard package paths
under `/var/lib/waydroid/data/` were absent. No sibling apps or unrelated
private directories were scanned.

The ADB executable is installed. A process check found no matching ADB
process; a scoped host socket check found no listener on default port 5037.
The sandbox initially denied the socket check; the required read-only host
escalation succeeded. In this initial pass, `adb devices` was not invoked
because it can start a server. The prior serial was historical, and no current
device state was queried during that pass. The game and Waydroid services
were not started or changed.

[Full extension evidence](placement-waydroid-cache-search.json) preserves
the searched roots, permission errors and socket observations. **The host
package cache remains inaccessible, so these results do not establish its
absence.** No authentic candidate was available to hand to the Rust decoder.

## Final authorized local ADB route

The root agent attempted read-only `sudo -n rg` over the four exact game
package roots and reported that sudo requires a password; no files were read
and no password was requested. A final alternate access pass was explicitly
authorized, including incidental startup of ADB's host daemon.

`waydroid status` reported `Session: STOPPED`. A read-only host service check
reported `waydroid-container` active. The approved `adb devices -l` invocation
started the host ADB daemon and listed no attached devices. Only the prior
task-known local endpoint `192.168.240.112:5555` was tried, with
`timeout 12 adb connect`; it returned no output and timed out with exit 124.
A final `adb devices -l` still listed no attached devices.

The game, container, graphical session and system services were not started
or changed. No package data, unrelated app data or unknown host was accessed.
The known cache routes are exhausted for this scope. The same
[retained JSON](placement-waydroid-cache-search.json) contains the initial
observations and the final alternate-access results. Cache contents remain
unknown because host traversal and the local ADB transport are unavailable.
