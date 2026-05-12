#include "ThemeController.h"

#include <QApplication>
#include <QGuiApplication>
#include <QPalette>
#include <QSettings>
#include <QStyleHints>

ThemeController::ThemeController(QObject *parent)
    : QObject(parent)
{
    load();
    updateDark();

    if (auto *app = qobject_cast<QGuiApplication *>(QCoreApplication::instance())) {
        connect(app->styleHints(), &QStyleHints::colorSchemeChanged, this, &ThemeController::updateDark);
    }
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
    applyPalette();
    emit darkChanged();
}

void ThemeController::applyPalette()
{
    QPalette p;
    if (m_dark) {
        p.setColor(QPalette::Window, QColor(0x12, 0x11, 0x0f));
        p.setColor(QPalette::WindowText, QColor(0xec, 0xe6, 0xd9));
        p.setColor(QPalette::Base, QColor(0x1f, 0x1d, 0x1a));
        p.setColor(QPalette::AlternateBase, QColor(0x28, 0x25, 0x21));
        p.setColor(QPalette::ToolTipBase, QColor(0x28, 0x25, 0x21));
        p.setColor(QPalette::ToolTipText, QColor(0xec, 0xe6, 0xd9));
        p.setColor(QPalette::Text, QColor(0xec, 0xe6, 0xd9));
        p.setColor(QPalette::Button, QColor(0x1f, 0x1d, 0x1a));
        p.setColor(QPalette::ButtonText, QColor(0xec, 0xe6, 0xd9));
        p.setColor(QPalette::BrightText, QColor(0xd1, 0x41, 0x1a));
        p.setColor(QPalette::Link, QColor(0x4c, 0xab, 0x7e));
        p.setColor(QPalette::Highlight, QColor(0x4c, 0xab, 0x7e));
        p.setColor(QPalette::HighlightedText, QColor(0x12, 0x11, 0x0f));
    } else {
        p.setColor(QPalette::Window, QColor(0xec, 0xe6, 0xd9));
        p.setColor(QPalette::WindowText, QColor(0x16, 0x14, 0x0f));
        p.setColor(QPalette::Base, QColor(0xf8, 0xf4, 0xea));
        p.setColor(QPalette::AlternateBase, QColor(0xe3, 0xdc, 0xcb));
        p.setColor(QPalette::ToolTipBase, QColor(0xe3, 0xdc, 0xcb));
        p.setColor(QPalette::ToolTipText, QColor(0x16, 0x14, 0x0f));
        p.setColor(QPalette::Text, QColor(0x16, 0x14, 0x0f));
        p.setColor(QPalette::Button, QColor(0xf8, 0xf4, 0xea));
        p.setColor(QPalette::ButtonText, QColor(0x16, 0x14, 0x0f));
        p.setColor(QPalette::BrightText, QColor(0xc6, 0x3b, 0x13));
        p.setColor(QPalette::Link, QColor(0x1f, 0x7a, 0x4d));
        p.setColor(QPalette::Highlight, QColor(0x1f, 0x7a, 0x4d));
        p.setColor(QPalette::HighlightedText, QColor(0xf8, 0xf4, 0xea));
    }
    qApp->setPalette(p);
}
