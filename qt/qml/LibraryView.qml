import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

// Media library: searchable clip grid (left) + inspector (right).
SplitView {
    id: library
    orientation: Qt.Horizontal

    // Recomputed when the library reloads (count) or the search text changes.
    readonly property var rows: {
        appController.clipLibrary.count // dependency
        return appController.clipLibrary.items(searchField.text)
    }

    handle: Rectangle {
        implicitWidth: 6
        color: SplitHandle.pressed ? Theme.accent.brand
            : (SplitHandle.hovered ? Theme.surface.hover : Theme.surface.backdrop)
        Behavior on color { ColorAnimation { duration: Theme.motionFast } }
        Rectangle { anchors.centerIn: parent; width: 1; height: parent.height; color: Theme.border.base }
    }

    Rectangle {
        SplitView.fillWidth: true
        SplitView.minimumWidth: 380
        color: Theme.surface.base

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            // Toolbar: search + count
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 52
                color: Theme.surface.panel
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space4
                    anchors.rightMargin: Theme.space4
                    spacing: Theme.space2
                    Icon { name: "search"; size: 15; color: Theme.text.faint }
                    DenseTextField {
                        id: searchField
                        Layout.fillWidth: true
                        Layout.maximumWidth: 360
                        placeholderText: "Search clips by name or codec…"
                    }
                    Item { Layout.fillWidth: true }
                    Text {
                        text: library.rows.length + " of " + appController.clipLibrary.count + " clips"
                        color: Theme.text.faint
                        font.family: Theme.fontMono
                        font.pixelSize: Theme.textCaption
                    }
                }
                Divider { anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom }
            }

            // Empty state
            EmptyState {
                Layout.fillWidth: true
                Layout.fillHeight: true
                visible: appController.clipLibrary.count === 0
                iconName: "grid"
                title: "No clips in the library yet"
                subtitle: !appController.ffprobeAvailable
                    ? "Install FFmpeg so clip metadata can be extracted, then run an offload to populate the library."
                    : (!appController.extractMetadata
                        ? "Enable metadata extraction, then run an offload to populate the library with clips and metadata."
                        : "Run an offload to populate the library with clips, thumbnails and metadata.")

                QuietButton {
                    Layout.alignment: Qt.AlignHCenter
                    visible: appController.ffprobeAvailable && !appController.extractMetadata
                    text: "Enable metadata extraction"
                    iconName: "check"
                    variant: "primary"
                    onClicked: appController.extractMetadata = true
                }
            }

            // Clip grid
            GridView {
                id: grid
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.margins: Theme.space3
                visible: appController.clipLibrary.count > 0
                clip: true
                cellWidth: 196
                cellHeight: 178
                model: library.rows
                cacheBuffer: 720
                currentIndex: -1
                ScrollBar.vertical: ScrollBar {}

                delegate: Item {
                    required property int index
                    required property var modelData
                    width: grid.cellWidth
                    height: grid.cellHeight
                    ClipCard {
                        anchors.fill: parent
                        anchors.margins: Theme.space1
                        fileName: modelData.fileName
                        mediaKind: modelData.mediaKind
                        durationText: modelData.duration
                        metaLine: {
                            var parts = []
                            if (modelData.codec) parts.push(modelData.codec)
                            if (modelData.resolution) parts.push(modelData.resolution)
                            if (modelData.fps) parts.push(modelData.fps)
                            return parts.length > 0 ? parts.join(" · ") : modelData.sizeText
                        }
                        thumbSource: (modelData.hash && String(modelData.hash).length > 0
                                      && appController.clipLibrary.sourcePath.length > 0)
                            ? "image://clipthumb/" + modelData.algorithm + "/" + modelData.hash + "/"
                                + encodeURIComponent(appController.clipLibrary.sourcePath + "/" + modelData.relPath)
                            : ""
                        selected: grid.currentIndex === index
                        Accessible.role: Accessible.ListItem
                        Accessible.name: modelData.fileName + ", " + metaLine
                        onClicked: grid.currentIndex = index
                    }
                }
            }
        }
    }

    InspectorPanel {
        SplitView.preferredWidth: 324
        SplitView.minimumWidth: 264
        SplitView.maximumWidth: 460
        sourcePath: appController.clipLibrary.sourcePath
        clip: (grid.currentIndex >= 0 && grid.currentIndex < library.rows.length)
            ? library.rows[grid.currentIndex]
            : null
    }
}
