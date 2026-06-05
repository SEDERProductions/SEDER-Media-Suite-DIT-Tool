#pragma once

#include "DestinationListModel.h"
#include "DitOffloadWorker.h"
#include "MediaListModel.h"
#include "ThemeController.h"
#include "ThumbnailWorker.h"

#include <QElapsedTimer>
#include <QObject>
#include <QPointer>
#include <QStringList>
#include <QThread>

class SettingsStore;

class AppController final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString sourcePath READ sourcePath WRITE setSourcePath NOTIFY sourcePathChanged)
    Q_PROPERTY(DestinationListModel *destinationModel READ destinationModel CONSTANT)
    Q_PROPERTY(MediaListModel *mediaModel READ mediaModel CONSTANT)
    Q_PROPERTY(bool mediaScanning READ mediaScanning NOTIFY mediaScanningChanged)
    Q_PROPERTY(QString projectName READ projectName WRITE setProjectName NOTIFY projectNameChanged)
    Q_PROPERTY(QString shootDate READ shootDate WRITE setShootDate NOTIFY shootDateChanged)
    Q_PROPERTY(bool shootDateValid READ shootDateValid NOTIFY shootDateChanged)
    Q_PROPERTY(QString cardName READ cardName WRITE setCardName NOTIFY cardNameChanged)
    Q_PROPERTY(QString cameraId READ cameraId WRITE setCameraId NOTIFY cameraIdChanged)
    Q_PROPERTY(QString ignorePatterns READ ignorePatterns WRITE setIgnorePatterns NOTIFY ignorePatternsChanged)
    Q_PROPERTY(bool ignoreHiddenSystem READ ignoreHiddenSystem WRITE setIgnoreHiddenSystem NOTIFY ignoreHiddenSystemChanged)
    Q_PROPERTY(bool verifyAfterCopy READ verifyAfterCopy WRITE setVerifyAfterCopy NOTIFY verifyAfterCopyChanged)
    Q_PROPERTY(bool skipExisting READ skipExisting WRITE setSkipExisting NOTIFY skipExistingChanged)
    Q_PROPERTY(bool generateReport READ generateReport WRITE setGenerateReport NOTIFY generateReportChanged)
    Q_PROPERTY(QString checksumAlgorithm READ checksumAlgorithm WRITE setChecksumAlgorithm NOTIFY checksumAlgorithmChanged)
    Q_PROPERTY(bool extractMetadata READ extractMetadata WRITE setExtractMetadata NOTIFY extractMetadataChanged)
    Q_PROPERTY(bool ffprobeAvailable READ ffprobeAvailable CONSTANT)
    Q_PROPERTY(bool ffmpegAvailable READ ffmpegAvailable CONSTANT)
    Q_PROPERTY(bool canExportMetadataJson READ canExportMetadataJson NOTIFY canExportMetadataJsonChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(double overallProgress READ overallProgress NOTIFY overallProgressChanged)
    Q_PROPERTY(QString transferSpeed READ transferSpeed NOTIFY rateChanged)
    Q_PROPERTY(QString etaText READ etaText NOTIFY rateChanged)
    Q_PROPERTY(QString statusText READ statusText NOTIFY statusTextChanged)
    Q_PROPERTY(QString currentFile READ currentFile NOTIFY currentFileChanged)
    Q_PROPERTY(QStringList logLines READ logLines NOTIFY logLinesChanged)
    Q_PROPERTY(bool canExport READ canExport NOTIFY exportStateChanged)
    Q_PROPERTY(bool canExportMhl READ canExportMhl NOTIFY canExportMhlChanged)
    Q_PROPERTY(QString finalStatus READ finalStatus NOTIFY summaryChanged)
    Q_PROPERTY(bool verificationPerformed READ verificationPerformed NOTIFY summaryChanged)
    Q_PROPERTY(quint64 totalFiles READ totalFiles NOTIFY summaryChanged)
    Q_PROPERTY(quint64 totalSize READ totalSize NOTIFY summaryChanged)
    Q_PROPERTY(bool pass READ pass NOTIFY summaryChanged)
    Q_PROPERTY(QString appVersion READ appVersion CONSTANT)

public:
    explicit AppController(SettingsStore *settings = nullptr, QObject *parent = nullptr);
    ~AppController() override;

    QString appVersion() const;

    QString sourcePath() const;
    void setSourcePath(const QString &value);
    DestinationListModel *destinationModel() const;
    MediaListModel *mediaModel() const;
    bool mediaScanning() const;
    QString projectName() const;
    void setProjectName(const QString &value);
    QString shootDate() const;
    void setShootDate(const QString &value);
    bool shootDateValid() const;
    QString cardName() const;
    void setCardName(const QString &value);
    QString cameraId() const;
    void setCameraId(const QString &value);
    QString ignorePatterns() const;
    void setIgnorePatterns(const QString &value);
    bool ignoreHiddenSystem() const;
    void setIgnoreHiddenSystem(bool value);
    bool verifyAfterCopy() const;
    void setVerifyAfterCopy(bool value);
    bool skipExisting() const;
    void setSkipExisting(bool value);
    bool generateReport() const;
    void setGenerateReport(bool value);
    QString checksumAlgorithm() const;
    void setChecksumAlgorithm(const QString &value);
    bool extractMetadata() const;
    void setExtractMetadata(bool value);
    bool ffprobeAvailable() const;
    bool ffmpegAvailable() const;
    bool canExportMetadataJson() const;
    bool busy() const;
    double overallProgress() const;
    QString transferSpeed() const;
    QString etaText() const;
    QString statusText() const;
    QString currentFile() const;
    QStringList logLines() const;
    bool canExport() const;
    bool canExportMhl() const;
    QString finalStatus() const;
    bool verificationPerformed() const;
    quint64 totalFiles() const;
    quint64 totalSize() const;
    bool pass() const;

    Q_INVOKABLE void chooseSourceFolder();
    Q_INVOKABLE void loadSourceMedia();
    Q_INVOKABLE void clearSourceMedia();
    Q_INVOKABLE QString formatBreakdownJson() const;
    Q_INVOKABLE void addDestinationFolder();
    Q_INVOKABLE void addSourceFromPath(const QString &path);
    Q_INVOKABLE void addDestinationFromPath(const QString &path);
    Q_INVOKABLE void copyDestinationPath(int sourceIndex);
    Q_INVOKABLE void syncDestinationPaths();
    Q_INVOKABLE void removeDestination(int index);
    Q_INVOKABLE void renameDestination(int index, const QString &label);
    Q_INVOKABLE void startOffload();
    Q_INVOKABLE void cancelOffload();
    Q_INVOKABLE void exportTxt();
    Q_INVOKABLE void exportCsv();
    Q_INVOKABLE void exportMhl();
    Q_INVOKABLE void exportMetadataJson();
    Q_INVOKABLE void exportAle();
    Q_INVOKABLE void clearLog();
    Q_INVOKABLE void copyLog();
    Q_INVOKABLE QString formatBytes(quint64 value) const;
    Q_INVOKABLE void applyDefaultsFromSettings();
    Q_INVOKABLE QString previewDestinationTemplate(const QString &basePath) const;

signals:
    void sourcePathChanged();
    void mediaScanningChanged();
    void projectNameChanged();
    void shootDateChanged();
    void cardNameChanged();
    void cameraIdChanged();
    void ignorePatternsChanged();
    void ignoreHiddenSystemChanged();
    void verifyAfterCopyChanged();
    void skipExistingChanged();
    void generateReportChanged();
    void checksumAlgorithmChanged();
    void extractMetadataChanged();
    void canExportMetadataJsonChanged();
    void busyChanged();
    void overallProgressChanged();
    void rateChanged();
    void statusTextChanged();
    void currentFileChanged();
    void logLinesChanged();
    void exportStateChanged();
    void canExportMhlChanged();
    void summaryChanged();
    void exportSucceeded(const QString &path);
    void exportFailed(const QString &message);
    void offloadFailed(const QString &message);

public:
    enum class LogSeverity {
        Info,
        Warn,
        Error
    };
private:
    void appendLog(const QString &line, LogSeverity severity = LogSeverity::Info);
    void startThumbnailGeneration();
    void stopThumbnailGeneration();
    void updateTransferRate(quint64 bytesCompleted, quint64 bytesTotal);
    void resetTransferRate();
    static QString formatDuration(qint64 seconds);
    void setBusy(bool value);
    void setOverallProgress(double value);
    void setStatusText(const QString &value);
    void setCurrentFile(const QString &value);
    void setPass(bool value);
    void writeExport(const QString &caption, const QString &defaultName, const QString &contents);

    SettingsStore *m_settings = nullptr;
    DestinationListModel *m_destinationModel = nullptr;
    MediaListModel *m_mediaModel = nullptr;
    QString m_thumbCacheDir;
    bool m_mediaScanning = false;
    QPointer<ThumbnailWorker> m_thumbWorker;
    QPointer<QThread> m_thumbThread;
    QString m_sourcePath;
    QString m_projectName;
    QString m_shootDate;
    QString m_cardName;
    QString m_cameraId;
    QString m_ignorePatterns = QStringLiteral(".DS_Store, Thumbs.db, desktop.ini, .Spotlight-V100, .Trashes");
    bool m_ignoreHiddenSystem = true;
    bool m_verifyAfterCopy = true;
    bool m_skipExisting = false;
    bool m_generateReport = true;
    QString m_checksumAlgorithm = QStringLiteral("BLAKE3");
    bool m_extractMetadata = false;
    QString m_metadataJsonExport;
    QString m_aleExport;
    bool m_busy = false;
    double m_overallProgress = 0.0;
    QElapsedTimer m_rateTimer;
    quint64 m_lastBytes = 0;
    double m_smoothedBytesPerSec = 0.0;
    QString m_transferSpeed;
    QString m_etaText;
    QString m_statusText = QStringLiteral("Ready for offload.");
    QString m_currentFile;
    QStringList m_logLines;
    bool m_canExport = false;
    bool m_canExportMhl = false;
    QString m_finalStatus = QStringLiteral("FAIL");
    bool m_verificationPerformed = false;
    quint64 m_totalFiles = 0;
    quint64 m_totalSize = 0;
    bool m_pass = false;
    DitOffloadWorker *m_activeWorker = nullptr;
    QString m_txtExport;
    QString m_csvExport;
    QString m_mhlExport;
    QVector<quint64> m_prevDestFilesCompleted;
};
