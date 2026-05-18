#include "SettingsStore.h"

#include <QSettings>

namespace {
constexpr const char *kOrganization = "Seder Productions";
constexpr const char *kApplication = "SEDER Media Suite";
constexpr const char *kKeyRecentSources = "dit/recentSources";
constexpr const char *kKeyRecentDestinations = "dit/recentDestinations";
constexpr const char *kKeyDefaultIgnorePatterns = "dit/defaults/ignorePatterns";
constexpr const char *kKeyDefaultVerifyAfterCopy = "dit/defaults/verifyAfterCopy";
constexpr const char *kKeyDefaultIgnoreHiddenSystem = "dit/defaults/ignoreHiddenSystem";
constexpr const char *kKeyDefaultSkipExisting = "dit/defaults/skipExisting";
constexpr const char *kKeyDefaultGenerateReport = "dit/defaults/generateReport";
constexpr const char *kKeyDefaultChecksumAlgorithm = "dit/defaults/checksumAlgorithm";
constexpr const char *kKeyDestinationTemplate = "dit/defaults/destinationTemplate";
constexpr const char *kKeyWindowGeometry = "dit/window/geometry";
constexpr const char *kKeyLastProject = "dit/lastMetadata/projectName";
constexpr const char *kKeyLastShootDate = "dit/lastMetadata/shootDate";
constexpr const char *kKeyLastCardName = "dit/lastMetadata/cardName";
constexpr const char *kKeyLastCameraId = "dit/lastMetadata/cameraId";

QSettings settings()
{
    return QSettings(QString::fromLatin1(kOrganization), QString::fromLatin1(kApplication));
}
} // namespace

SettingsStore::SettingsStore(QObject *parent)
    : QObject(parent)
    , m_defaultIgnorePatterns(QString::fromLatin1(kDefaultIgnorePatternsValue))
{
    load();
}

void SettingsStore::load()
{
    auto s = settings();
    m_recentSources = s.value(QString::fromLatin1(kKeyRecentSources)).toStringList();
    m_recentDestinations = s.value(QString::fromLatin1(kKeyRecentDestinations)).toStringList();
    m_defaultIgnorePatterns = s.value(QString::fromLatin1(kKeyDefaultIgnorePatterns),
                                      QString::fromLatin1(kDefaultIgnorePatternsValue)).toString();
    m_defaultVerifyAfterCopy = s.value(QString::fromLatin1(kKeyDefaultVerifyAfterCopy), true).toBool();
    m_defaultIgnoreHiddenSystem = s.value(QString::fromLatin1(kKeyDefaultIgnoreHiddenSystem), true).toBool();
    m_defaultSkipExisting = s.value(QString::fromLatin1(kKeyDefaultSkipExisting), false).toBool();
    m_defaultGenerateReport = s.value(QString::fromLatin1(kKeyDefaultGenerateReport), true).toBool();
    m_defaultChecksumAlgorithm = s.value(QString::fromLatin1(kKeyDefaultChecksumAlgorithm),
                                          QStringLiteral("BLAKE3")).toString();
    m_destinationTemplate = s.value(QString::fromLatin1(kKeyDestinationTemplate)).toString();
    m_lastProjectName = s.value(QString::fromLatin1(kKeyLastProject)).toString();
    m_lastShootDate = s.value(QString::fromLatin1(kKeyLastShootDate)).toString();
    m_lastCardName = s.value(QString::fromLatin1(kKeyLastCardName)).toString();
    m_lastCameraId = s.value(QString::fromLatin1(kKeyLastCameraId)).toString();
}

QStringList SettingsStore::recentSources() const { return m_recentSources; }
QStringList SettingsStore::recentDestinations() const { return m_recentDestinations; }
QString SettingsStore::defaultIgnorePatterns() const { return m_defaultIgnorePatterns; }
bool SettingsStore::defaultVerifyAfterCopy() const { return m_defaultVerifyAfterCopy; }
bool SettingsStore::defaultIgnoreHiddenSystem() const { return m_defaultIgnoreHiddenSystem; }
bool SettingsStore::defaultSkipExisting() const { return m_defaultSkipExisting; }
bool SettingsStore::defaultGenerateReport() const { return m_defaultGenerateReport; }
QString SettingsStore::defaultChecksumAlgorithm() const { return m_defaultChecksumAlgorithm; }
QString SettingsStore::destinationTemplate() const { return m_destinationTemplate; }

QString SettingsStore::lastProjectName() const { return m_lastProjectName; }
QString SettingsStore::lastShootDate() const { return m_lastShootDate; }
QString SettingsStore::lastCardName() const { return m_lastCardName; }
QString SettingsStore::lastCameraId() const { return m_lastCameraId; }

void SettingsStore::setDefaultIgnorePatterns(const QString &value)
{
    if (m_defaultIgnorePatterns == value) return;
    m_defaultIgnorePatterns = value;
    settings().setValue(QString::fromLatin1(kKeyDefaultIgnorePatterns), value);
    emit defaultIgnorePatternsChanged();
}

void SettingsStore::setDefaultVerifyAfterCopy(bool value)
{
    if (m_defaultVerifyAfterCopy == value) return;
    m_defaultVerifyAfterCopy = value;
    settings().setValue(QString::fromLatin1(kKeyDefaultVerifyAfterCopy), value);
    emit defaultVerifyAfterCopyChanged();
}

void SettingsStore::setDefaultIgnoreHiddenSystem(bool value)
{
    if (m_defaultIgnoreHiddenSystem == value) return;
    m_defaultIgnoreHiddenSystem = value;
    settings().setValue(QString::fromLatin1(kKeyDefaultIgnoreHiddenSystem), value);
    emit defaultIgnoreHiddenSystemChanged();
}

void SettingsStore::setDefaultSkipExisting(bool value)
{
    if (m_defaultSkipExisting == value) return;
    m_defaultSkipExisting = value;
    settings().setValue(QString::fromLatin1(kKeyDefaultSkipExisting), value);
    emit defaultSkipExistingChanged();
}

void SettingsStore::setDefaultGenerateReport(bool value)
{
    if (m_defaultGenerateReport == value) return;
    m_defaultGenerateReport = value;
    settings().setValue(QString::fromLatin1(kKeyDefaultGenerateReport), value);
    emit defaultGenerateReportChanged();
}

void SettingsStore::setDefaultChecksumAlgorithm(const QString &value)
{
    static const QStringList valid = {
        QStringLiteral("BLAKE3"), QStringLiteral("MD5"), QStringLiteral("SHA1"),
        QStringLiteral("XXH3-64"), QStringLiteral("XXH3-128")
    };
    const QString upper = value.trimmed().toUpper();
    const QString normalized = valid.contains(upper) ? upper : QStringLiteral("BLAKE3");
    if (m_defaultChecksumAlgorithm == normalized) return;
    m_defaultChecksumAlgorithm = normalized;
    settings().setValue(QString::fromLatin1(kKeyDefaultChecksumAlgorithm), normalized);
    emit defaultChecksumAlgorithmChanged();
}

void SettingsStore::setDestinationTemplate(const QString &value)
{
    const QString trimmed = value.trimmed();
    if (m_destinationTemplate == trimmed) return;
    m_destinationTemplate = trimmed;
    settings().setValue(QString::fromLatin1(kKeyDestinationTemplate), trimmed);
    emit destinationTemplateChanged();
}

QRect SettingsStore::windowGeometry() const
{
    return settings().value(QString::fromLatin1(kKeyWindowGeometry)).toRect();
}

void SettingsStore::setWindowGeometry(const QRect &rect)
{
    if (rect.width() <= 0 || rect.height() <= 0) return;
    settings().setValue(QString::fromLatin1(kKeyWindowGeometry), rect);
}

void SettingsStore::setLastProjectMetadata(const QString &project,
                                           const QString &shootDate,
                                           const QString &cardName,
                                           const QString &cameraId)
{
    m_lastProjectName = project;
    m_lastShootDate = shootDate;
    m_lastCardName = cardName;
    m_lastCameraId = cameraId;
    auto s = settings();
    s.setValue(QString::fromLatin1(kKeyLastProject), project);
    s.setValue(QString::fromLatin1(kKeyLastShootDate), shootDate);
    s.setValue(QString::fromLatin1(kKeyLastCardName), cardName);
    s.setValue(QString::fromLatin1(kKeyLastCameraId), cameraId);
}

QStringList SettingsStore::pushRecent(QStringList list, const QString &path)
{
    if (path.isEmpty()) return list;
    list.removeAll(path);
    list.prepend(path);
    while (list.size() > kMaxRecent) {
        list.removeLast();
    }
    return list;
}

void SettingsStore::rememberSource(const QString &path)
{
    const QStringList next = pushRecent(m_recentSources, path);
    if (next == m_recentSources) return;
    m_recentSources = next;
    settings().setValue(QString::fromLatin1(kKeyRecentSources), m_recentSources);
    emit recentSourcesChanged();
}

void SettingsStore::rememberDestination(const QString &path)
{
    const QStringList next = pushRecent(m_recentDestinations, path);
    if (next == m_recentDestinations) return;
    m_recentDestinations = next;
    settings().setValue(QString::fromLatin1(kKeyRecentDestinations), m_recentDestinations);
    emit recentDestinationsChanged();
}

void SettingsStore::resetDefaultsToFactory()
{
    setDefaultIgnorePatterns(QString::fromLatin1(kDefaultIgnorePatternsValue));
    setDefaultVerifyAfterCopy(true);
    setDefaultIgnoreHiddenSystem(true);
    setDefaultSkipExisting(false);
    setDefaultGenerateReport(true);
    setDefaultChecksumAlgorithm(QStringLiteral("BLAKE3"));
    setDestinationTemplate(QString());
}
