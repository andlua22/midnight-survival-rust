// JSON and file-identity bookkeeping only. Unity extraction is performed in Rust.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const evidence = __dirname;
const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const hash = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const imported = read(path.join(evidence, 'import.json'));
const repeated = read(path.join(evidence, 'repeat-import.json'));
const current = read(path.join(imported.package, 'manifest.json'));
const previous = read(path.join(imported.backup, 'manifest.json'));
assert.equal(current.schema_version, 4);
assert.equal(previous.schema_version, 3);
assert.deepEqual(current.meshes, previous.meshes);
assert.deepEqual(current.source_apk, previous.source_apk);
assert.deepEqual(current.geometry, previous.geometry);
assert.equal(hash(path.join(imported.package, 'manifest.json')), imported.manifest_sha256);
assert.equal(hash(path.join(imported.backup, 'manifest.json')), imported.previous_manifest_sha256);
assert.equal(repeated.already_imported, true);
assert.equal(repeated.backup, null);
assert.equal(repeated.previous_manifest_sha256, imported.manifest_sha256);
assert.equal(repeated.manifest_sha256, imported.manifest_sha256);
assert.deepEqual(repeated.collision_geometry, imported.collision_geometry);
assert.deepEqual(repeated.previously_unrepresented_entries, imported.previously_unrepresented_entries);
assert.deepEqual(repeated.source_bundle_hashes, imported.source_bundle_hashes);
assert.deepEqual(repeated.collision_meshes, imported.collision_meshes);
assert.equal(imported.preserved_files.length, 970);
assert.equal(new Set(imported.preserved_files.map(file => file.path)).size, 970);
assert.equal(repeated.preserved_files.length, 971);
assert.deepEqual(repeated.preserved_files.filter(file => file.path !== imported.collision_geometry.path), imported.preserved_files);
assert.deepEqual(repeated.preserved_files.find(file => file.path === imported.collision_geometry.path), imported.collision_geometry);

let bytes = 0;
for (const identity of imported.preserved_files) {
    for (const root of [imported.package, imported.backup]) {
        assert.equal(hash(path.join(root, identity.path)), identity.sha256);
    }
    bytes += fs.statSync(path.join(imported.package, identity.path)).size;
}
assert.equal(hash(path.join(imported.package, imported.collision_geometry.path)), imported.collision_geometry.sha256);
const report = {
    schema_version: 1,
    renderer_records_unchanged: Object.keys(current.meshes).length,
    source_apk_identity_unchanged: true,
    render_geometry_identity_unchanged: true,
    preserved_payloads: imported.preserved_files.length,
    current_and_backup_payload_hashes_checked: imported.preserved_files.length * 2,
    preserved_payload_bytes: bytes,
    all_payload_hashes_match: true,
    collision_archive_hash_matches: true,
    manifest_hashes_match: true,
    repeat_already_imported: true,
    repeat_created_backup: false,
    repeat_manifest_unchanged: true,
    repeat_source_capture_unchanged: true,
    repeat_preserved_payloads_including_collision: repeated.preserved_files.length,
};
fs.writeFileSync(path.join(evidence, 'preservation-check.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
