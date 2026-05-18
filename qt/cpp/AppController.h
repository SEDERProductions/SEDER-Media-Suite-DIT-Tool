#pragma once

#include "DestinationListModel.h"
#include "DitOffloadWorker.h"
#include "ThemeController.h"

#include <QObject>
#include <QStringList>

class SettingsStore;

class AppController final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString sourcePath READ sourcePath WRITE setSourcePath NOTIFY sourcePathChanged)
    Q_PROPERTY(DestinationListModel *destinationModel READ destinationModel CONSTANT)
    Q_PROPERTY(QString projectName READ projectName WRITE setProjectName NOTIFY projectNameChanged)
    Q_PROPERTY(QString shootDate READ shootDate WRITE setShootDate NOTIFY shootDateChanged)
    Q_PROPERTY(QString cardName READ cardName WRITE setCardName NOTIFY cardNameChanged)
    Q_PROPERTY(QString cameraId READ cameraId WRITE setCameraId NOTIFY cameraIdChanged)
    Q_PROPERTY(QString ignorePatterns READ ignorePatterns WRITE setIgnorePatterns NOTIFY ignorePatternsChanged)
    Q_PROPERTY(bool ignoreHiddenSystem READ ignoreHiddenSystem WRITE setIgnoreHiddenSystem NOTIFY ignoreHiddenSystemChanged)
    Q_PROPERTY(bool verifyAfterCopy READ verifyAfterCopy WRITE setVerifyAfterCopy NOTIFY verifyAfterCopyChanged)
    Q_PROPERTY(bool skipExisting READ skipExisting WRITE setSkipExisting NOTIFY skipExistingChanged)
    Q_PROPERTY(bool generateReport READ generateReport WRITE setGenerateReport NOTIFY generateReportChanged)
    Q_PROPERTY(QString checksumAlgorithm READ checksumAlgorithm WRITE setChecksumAlgorithm NOTIFY checksumAlgorithmChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(double overallProgress READ overallProgress NOTIFY overallProgressChanged)
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

    QString appVersion() const;

    QString sourcePath() const;
    void setSourcePath(const QString &value);
    DestinationListModel *destinationModel() const;
    QString projectName() const;
    void setProjectName(const QString &value);
    QString shootDate() const;
    void setShootDate(const QString &value);
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
    bool busy() const;
    double overallProgress() const;
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
    Q_INVOKABLE void addDestinationFolder();
    Q_INVOKABLE void addSourceFromPath(const QString &path);
    Q_INVOKABLE void addDestinationFromPath(const QString &path);
    Q_INVOKABLE void copyDestinationPath(int sourceIndex);
    Q_INVOKABLE void syncDestinationPaths();
    Q_INVOKABLE void removeDestination(int index);
    Q_INVOKABLE void startOffload();
    Q_INVOKABLE void cancelOffload();
    Q_INVOKABLE void exportTxt();
    Q_INVOKABLE void exportCsv();
    Q_INVOKABLE void exportMhl();
    Q_INVOKABLE void clearLog();
    Q_INVOKABLE void copyLog();
    Q_INVOKABLE QString formatBytes(quint64 value) const;
    Q_INVOKABLE void applyDefaultsFromSettings();

signals:
    void sourcePathChanged();
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
    void busyChanged();
    void overallProgressChanged();
    void statusTextChanged();
    void currentFileChanged();
    void logLinesChanged();
    void exportStateChanged();
    void canExportMhlChanged();
    void summaryChanged();

public:
    enum class LogSeverity {
        Info,
        Warn,
        Error
    };
private:
    void appendLog(const QString &line, LogSeverity severity = LogSeverity::Info);
    void setBusy(bool value);
    void setOverallProgress(double value);
    void setStatusText(const QString &value);
    void setCurrentFile(const QString &value);
    void setPass(bool value);
    void writeExport(const QString &caption, const QString &defaultName, const QString &contents);

    SettingsStore *m_settings = nullptr;
    DestinationListModel *m_destinationModel = nullptr;
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
    bool m_busy = false;
    double m_overallProgress = 0.0;
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
