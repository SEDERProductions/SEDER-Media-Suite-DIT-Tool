import QtQuick
import QtQuick.Controls

ComboBox {
    id: control
    readonly property bool dark: themeController.dark
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color red: dark ? "#d1411a" : "#c63b13"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    font.family: sans
    font.pixelSize: 12
    contentItem: Text {
        leftPadding: 8
        rightPadding: 8
        text: control.displayText
        color: ink
        font: control.font
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    background: Rectangle {
        color: panelAlt
        border.color: line
        radius: 4
    }
    popup: Popup {
        y: control.height
        width: control.width
        padding: 1
        background: Rectangle {
            color: panelAlt
            border.color: line
            radius: 4
        }
        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: control.model
            currentIndex: control.currentIndex
            delegate: ItemDelegate {
                width: ListView.view.width
                contentItem: Text {
                    text: modelData
                    color: ink
                    font: control.font
                    elide: Text.ElideRight
                    verticalAlignment: Text.AlignVCenter
                }
                highlighted: control.highlightedIndex === index
                background: Rectangle {
                    color: highlighted ? red : "transparent"
                    radius: 2
                }
            }
            ScrollBar.vertical: ScrollBar {}
        }
    }
}
