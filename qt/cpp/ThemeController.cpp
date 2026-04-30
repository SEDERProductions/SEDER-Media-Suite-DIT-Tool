#include "ThemeController.h"

#include <QApplication>
#include <QPalette>
#include <QSettings>

ThemeController::ThemeController(QObject *parent)
    : QObject(parent)
{
    load();
    updateDark();
}

QString ThemeController::preference() const
{
    return m_preference;
}

void ThemeController::setPreference(const QString &preference)
{
    const QString normalized = preference == QStringLiteral("dark") || preference == QStringLiteral("light")
        ? preference
        : QStringLiteral("system");
    if (m_preference == normalized) {
        return;
    }
    m_preference = normalized;
    save();
    updateDark();
    emit preferenceChanged();
}

bool ThemeController::dark() const
{
    return m_dark;
}

void ThemeController::load()
{
    QSettings settings(QStringLiteral("Seder Productions"), QStringLiteral("SEDER Media Suite"));
    m_preference = settings.value(QStringLiteral("dit/theme"), QStringLiteral("system")).toString();
}

void ThemeController::save()
{
    QSettings settings(QStringLiteral("Seder Productions"), QStringLiteral("SEDER Media Suite"));
    settings.setValue(QStringLiteral("dit/theme"), m_preference);
}

void ThemeController::updateDark()
{
    bool next = false;
    if (m_preference == QStringLiteral("dark")) {
        next = true;
    } else if (m_preference == QStringLiteral("light")) {
        next = false;
    } else {
        next = qApp->palette().color(QPalette::Window).lightness() < 128;
    }
    if (m_dark == next) {
        return;
    }
    m_dark = next;
    emit darkChanged();
}
