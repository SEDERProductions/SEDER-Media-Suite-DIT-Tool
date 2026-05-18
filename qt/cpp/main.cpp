#include "AppController.h"
#include "DestinationListModel.h"
#include "SettingsStore.h"
#include "ThemeController.h"

#include <QApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickWindow>
#include <QRect>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    app.setOrganizationName(QStringLiteral("Seder Productions"));
    app.setApplicationName(QStringLiteral("SEDER Media Suite DIT"));

    SettingsStore settingsStore;
    ThemeController themeController;
    AppController appController(&settingsStore);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("appController"), &appController);
    engine.rootContext()->setContextProperty(QStringLiteral("themeController"), &themeController);
    engine.rootContext()->setContextProperty(QStringLiteral("settingsStore"), &settingsStore);

#if QT_VERSION >= QT_VERSION_CHECK(6, 5, 0)
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        [] { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
    engine.loadFromModule(QStringLiteral("SederDit"), QStringLiteral("Main"));
#else
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreated,
        &app,
        [](QObject *obj, const QUrl &) {
            if (!obj) QCoreApplication::exit(-1);
        },
        Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral("qrc:/SederDit/qml/Main.qml")));
#endif

    if (!engine.rootObjects().isEmpty()) {
        if (auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().first())) {
            const QRect saved = settingsStore.windowGeometry();
            if (saved.width() > 0 && saved.height() > 0) {
                window->setGeometry(saved);
            }
            QObject::connect(&app, &QGuiApplication::aboutToQuit, &settingsStore, [window, &settingsStore] {
                settingsStore.setWindowGeometry(window->geometry());
            });
        }
    }

    return app.exec();
}
