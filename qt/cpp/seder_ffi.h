#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SederDestinationConfig {
    const char *path;
    const char *label;
} SederDestinationConfig;

typedef struct SederOffloadRequest {
    const char *source_path;
    const SederDestinationConfig *destinations;
    size_t destination_count;
    const char *project_name;
    const char *shoot_date;
    const char *card_name;
    const char *camera_id;
    const char *ignore_patterns;
    uint8_t ignore_hidden_system;
    uint8_t verify_after_copy;
    uint8_t sync_writes;
    uint8_t skip_existing;
    uint8_t generate_report;
    uint8_t *cancel_token;
    /* NUL-terminated algorithm name: BLAKE3 / MD5 / SHA1 / XXH3-64 /
     * XXH3-128. Pass NULL to use the default (BLAKE3). */
    const char *checksum_algorithm;
    /* When non-zero, ffprobe is invoked for each recognised media file
     * and the result is included in the metadata JSON sidecar export.
     * No-op if ffprobe isn't available on the host. */
    uint8_t extract_metadata;
} SederOffloadRequest;

typedef struct SederDestinationProgress {
    uint32_t state;
    uint64_t files_completed;
    uint64_t files_total;
    uint64_t bytes_completed;
    uint64_t bytes_total;
    const char *current_file;
    uint32_t last_status;
    const char *error;
} SederDestinationProgress;

typedef struct SederOffloadProgress {
    const char *phase;
    uint64_t overall_files_completed;
    uint64_t overall_files_total;
    uint64_t overall_bytes_completed;
    uint64_t overall_bytes_total;
    const char *current_file;
    const char *warning;
    const SederDestinationProgress *destinations;
    size_t destination_count;
} SederOffloadProgress;

typedef void (*SederOffloadProgressCallback)(
    const SederOffloadProgress *progress,
    void *user_data
);

typedef struct OffloadReportHandle OffloadReportHandle;

OffloadReportHandle *seder_offload_start(
    const SederOffloadRequest *request,
    SederOffloadProgressCallback callback,
    void *user_data,
    char **error_out
);

void seder_report_free(OffloadReportHandle *handle);
void seder_string_free(char *ptr);
const char *seder_report_export_txt(OffloadReportHandle *handle);
const char *seder_report_export_csv(OffloadReportHandle *handle);
const char *seder_report_export_mhl(OffloadReportHandle *handle);
uint8_t seder_report_summary(
    OffloadReportHandle *handle,
    uint64_t *total_files_out,
    uint64_t *total_size_out,
    size_t *dest_count_out
);
uint8_t seder_report_dest_state(
    OffloadReportHandle *handle,
    size_t dest_index,
    uint32_t *state_out,
    uint64_t *files_copied_out,
    uint64_t *files_verified_out,
    uint64_t *files_failed_out,
    uint64_t *bytes_copied_out
);

uint8_t seder_report_verification_performed(OffloadReportHandle *handle);

/* Borrowed pointer to the JSON sidecar describing each file plus its
 * ffprobe metadata. Empty when extract_metadata was disabled. */
const char *seder_report_export_metadata_json(OffloadReportHandle *handle);

/* Borrowed pointer to the ALE (Avid Log Exchange) sidecar. Always
 * populated, but most useful when extract_metadata is on so per-clip
 * timecode and FPS columns are filled in. */
const char *seder_report_export_ale(OffloadReportHandle *handle);

/* 1 if the path resolves to an LTFS-mounted volume, 0 otherwise. */
uint8_t seder_is_ltfs_volume(const char *path);

/* Compare two MAJOR.MINOR.PATCH version strings (optionally prefixed
 * with "v"). Returns 1 if `latest` is strictly newer than `current`,
 * 0 otherwise (including on parse failure). */
uint8_t seder_version_is_newer(const char *current, const char *latest);

/* Crash-recovery checkpoint helpers. The JSON schema is opaque to
 * the FFI — pass through whatever the caller serialized.
 *   save: returns 1 on success, 0 on failure.
 *   load: returns a heap-allocated JSON string (free with
 *         seder_string_free) or NULL if no checkpoint exists.
 *   clear: idempotent delete; returns 1 on success. */
uint8_t seder_checkpoint_save(const char *state_dir, const char *checkpoint_json);
char *seder_checkpoint_load(const char *state_dir);
uint8_t seder_checkpoint_clear(const char *state_dir);

/* Transcode `media` using a named preset (PRORES / H264 / DNXHR). The
 * returned string is heap-allocated; free with seder_string_free.
 * Returns NULL on failure (missing ffmpeg, unknown preset, ffmpeg
 * non-zero exit). */
char *seder_generate_proxy(
    const char *media_path,
    const char *proxies_root,
    const char *preset_name);

/* 1 if ffprobe is discoverable on this host, 0 otherwise. */
uint8_t seder_ffprobe_available(void);
/* 1 if ffmpeg is discoverable on this host, 0 otherwise. */
uint8_t seder_ffmpeg_available(void);

/* Extract a thumbnail JPEG for the given media file into the cache
 * directory, content-addressed by (algorithm, hash). Returns the
 * heap-allocated absolute path on success, NULL on failure. The caller
 * frees with seder_string_free. */
char *seder_extract_thumbnail(
    const char *media_path,
    const char *cache_dir,
    const char *algorithm,
    const char *hash);

/* Expand a destination template like "{project}/{date}/{card}". The
 * returned string is heap-allocated; release it with seder_string_free.
 * Returns NULL on failure. */
char *seder_expand_template(
    const char *template_str,
    const char *project_name,
    const char *shoot_date,
    const char *card_name,
    const char *camera_id);

#ifdef __cplusplus
}
#endif
