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
    SederDestinationConfig *destinations;
    size_t destination_count;
    const char *project_name;
    const char *shoot_date;
    const char *card_name;
    const char *camera_id;
    const char *ignore_patterns;
    uint8_t ignore_hidden_system;
    uint8_t verify_after_copy;
    volatile uint8_t *cancel_token;
} SederOffloadRequest;

typedef struct SederDestinationProgress {
    uint32_t state;
    uint64_t files_completed;
    uint64_t files_total;
    uint64_t bytes_completed;
    uint64_t bytes_total;
    const char *current_file;
    const char *error;
} SederDestinationProgress;

typedef struct SederOffloadProgress {
    const char *phase;
    uint64_t overall_files_completed;
    uint64_t overall_files_total;
    uint64_t overall_bytes_completed;
    uint64_t overall_bytes_total;
    const char *current_file;
    SederDestinationProgress *destinations;
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

#ifdef __cplusplus
}
#endif
