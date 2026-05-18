#pragma once

#include <QObject>
#include <QRect>
#include <QString>
#include <QStringList>

class SettingsStore final : public QObject {
    Q_OBJECT
    Q_PROPERTY(QStringList recentSources READ recentSources NOTIFY recentSourcesChanged)
    Q_PROPERTY(QStringList recentDestinations READ recentDestinations NOTIFY recentDestinationsChanged)
    Q_PROPERTY(QString defaultIgnorePatterns READ defaultIgnorePatterns WRITE setDefaultIgnorePatterns NOTIFY defaultIgnorePatternsChanged)
    Q_PROPERTY(bool defaultVerifyAfterCopy READ defaultVerifyAfterCopy WRITE setDefaultVerifyAfterCopy NOTIFY defaultVerifyAfterCopyChanged)
    Q_PROPERTY(bool defaultIgnoreHiddenSystem READ defaultIgnoreHiddenSystem WRITE setDefaultIgnoreHiddenSystem NOTIFY defaultIgnoreHiddenSystemChanged)
    Q_PROPERTY(bool defaultSkipExisting READ defaultSkipExisting WRITE setDefaultSkipExisting NOTIFY defaultSkipExistingChanged)
    Q_PROPERTY(bool defaultGenerateReport READ defaultGenerateReport WRITE setDefaultGenerateReport NOTIFY defaultGenerateReportChanged)
    Q_PROPERTY(QString defaultChecksumAlgorithm READ defaultChecksumAlgorithm WRITE setDefaultChecksumAlgorithm NOTIFY defaultChecksumAlgorithmChanged)

public:
    static constexpr int kMaxRecent = 10;
    static constexpr const char *kDefaultIgnorePatternsValue =
        ".DS_Store, Thumbs.db, desktop.ini, .Spotlight-V100, .Trashes";

    explicit SettingsStore(QObject *parent = nullptr);

    QStringList recentSources() const;
    QStringList recentDestinations() const;
    QString defaultIgnorePatterns() const;
    void setDefaultIgnorePatterns(const QString &value);
    bool defaultVerifyAfterCopy() const;
    void setDefaultVerifyAfterCopy(bool value);
    bool defaultIgnoreHiddenSystem() const;
    void setDefaultIgnoreHiddenSystem(bool value);
    bool defaultSkipExisting() const;
    void setDefaultSkipExisting(bool value);
    bool defaultGenerateReport() const;
    void setDefaultGenerateReport(bool value);
    QString defaultChecksumAlgorithm() const;
    void setDefaultChecksumAlgorithm(const QString &value);

    QRect windowGeometry() const;
    void setWindowGeometry(const QRect &rect);

    QString lastProjectName() const;
    QString lastShootDate() const;
    QString lastCardName() const;
    QString lastCameraId() const;
    void setLastProjectMetadata(const QString &project,
                                const QString &shootDate,
                                const QString &cardName,
                                const QString &cameraId);

    Q_INVOKABLE void rememberSource(const QString &path);
    Q_INVOKABLE void rememberDestination(const QString &path);
    Q_INVOKABLE void resetDefaultsToFactory();

signals:
    void recentSourcesChanged();
    void recentDestinationsChanged();
    void defaultIgnorePatternsChanged();
    void defaultVerifyAfterCopyChanged();
    void defaultIgnoreHiddenSystemChanged();
    void defaultSkipExistingChanged();
    void defaultGenerateReportChanged();
    void defaultChecksumAlgorithmChanged();

private:
    void load();
    static QStringList pushRecent(QStringList list, const QString &path);

    QStringList m_recentSources;
    QStringList m_recentDestinations;
    QString m_defaultIgnorePatterns;
    bool m_defaultVerifyAfterCopy = true;
    bool m_defaultIgnoreHiddenSystem = true;
    bool m_defaultSkipExisting = false;
    bool m_defaultGenerateReport = true;
    QString m_defaultChecksumAlgorithm = QStringLiteral("BLAKE3");
    QString m_lastProjectName;
    QString m_lastShootDate;
    QString m_lastCardName;
    QString m_lastCameraId;
};
