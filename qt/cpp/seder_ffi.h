#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SEDER_COMPARE_PATH_SIZE 0u
#define SEDER_COMPARE_PATH_SIZE_MODIFIED 1u
#define SEDER_COMPARE_PATH_SIZE_CHECKSUM 2u

#define SEDER_ROW_MATCHING 0
#define SEDER_ROW_CHANGED 1
#define SEDER_ROW_ONLY_IN_A 2
#define SEDER_ROW_ONLY_IN_B 3
#define SEDER_ROW_FOLDER_ONLY_IN_A 4
#define SEDER_ROW_FOLDER_ONLY_IN_B 5

typedef struct SederDitReportHandle SederDitReportHandle;

typedef struct SederDitRequest {
    const char *source_path;
    const char *destination_path;
    const char *project_name;
    const char *shoot_date;
    const char *card_name;
    const char *camera_id;
    const char *ignore_patterns;
    uint32_t compare_mode;
    uint8_t ignore_hidden_system;
} SederDitRequest;

typedef struct SederDitSummary {
    uint64_t only_a;
    uint64_t only_b;
    uint64_t changed;
    uint64_t matching;
    uint64_t total_files;
    uint64_t total_folders;
    uint64_t total_size;
    uint8_t pass;
    uint8_t mhl_available;
    uint32_t compare_mode;
} SederDitSummary;

#ifdef __cplusplus
static_assert(sizeof(SederDitSummary) == 64, "FFI struct SederDitSummary size mismatch: must match Rust padded size");
#endif

typedef void (*SederProgressCallback)(
    const char *phase,
    uint64_t processed_files,
    uint64_t processed_bytes,
    const char *status,
    void *user_data);

SederDitReportHandle *seder_dit_compare(
    const SederDitRequest *request,
    SederProgressCallback callback,
    void *user_data,
    char **error_out);

void seder_dit_report_free(SederDitReportHandle *handle);
void seder_string_free(char *value);

uint64_t seder_dit_report_row_count(const SederDitReportHandle *handle);
int seder_dit_report_row_status(const SederDitReportHandle *handle, uint64_t row);
uint8_t seder_dit_report_row_is_folder(const SederDitReportHandle *handle, uint64_t row);
const char *seder_dit_report_row_path(const SederDitReportHandle *handle, uint64_t row);
uint8_t seder_dit_report_row_size_a(const SederDitReportHandle *handle, uint64_t row, uint64_t *value_out);
uint8_t seder_dit_report_row_size_b(const SederDitReportHandle *handle, uint64_t row, uint64_t *value_out);
const char *seder_dit_report_row_checksum_a(const SederDitReportHandle *handle, uint64_t row);
const char *seder_dit_report_row_checksum_b(const SederDitReportHandle *handle, uint64_t row);
uint8_t seder_dit_report_summary(const SederDitReportHandle *handle, SederDitSummary *summary_out);
uint8_t seder_dit_report_mhl_available(const SederDitReportHandle *handle);
char *seder_dit_report_export_txt(const SederDitReportHandle *handle);
char *seder_dit_report_export_csv(const SederDitReportHandle *handle);
char *seder_dit_report_export_mhl(const SederDitReportHandle *handle);

#ifdef __cplusplus
}
#endif
