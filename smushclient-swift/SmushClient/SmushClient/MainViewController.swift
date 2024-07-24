import AppKit
import Cocoa

let inputAttrs = [
  NSAttributedString.Key.foregroundColor: NSColor.gray,
  NSAttributedString.Key.font: NSFont.monospacedSystemFont(
    ofSize: NSFont.systemFontSize, weight: .medium),
]

public class MainViewController: NSViewController, NSTextFieldDelegate, NSTextViewDelegate {
  @IBOutlet weak var inputField: NSTextField!
  @IBOutlet weak var scrollView: NSScrollView!
  @IBOutlet weak var splitView: NSSplitView!
  @IBOutlet weak var textView: NSTextView!

  let ansiColors = defaultAnsiColors()
  let bridge = RustMudBridge("discworld.atuin.net", 4242)
  var connectTask: Task<(), Never>?
  weak var textStorage: NSTextStorage!
  var willBreak = false
  let defaults = AppDefaults()

  override public func viewDidLoad() {
    textStorage = textView.textStorage
    inputField.delegate = self
    textView.delegate = self

    connect()
  }

  func connect() {
    connectTask = Task {
      do {
        try await bridge.connect()
        while true {
          let fragment = try await bridge.get_output()
          await receiveOutput(fragment)
        }
      } catch {
        print(error.localizedDescription)
      }
    }
  }

  func disconnect() {
    if let connectTask = connectTask {
      connectTask.cancel()
    }
    _ = bridge.disconnect()
  }

  public func control(
    _ control: NSControl, textView: NSTextView, doCommandBy commandSelector: Selector
  ) -> Bool {
    let input = inputField.stringValue
    inputField.stringValue = ""
    do {
      try sendInput(input)
    } catch {
      print(error.localizedDescription)
    }
    return true
  }

  public func textView(_ textView: NSTextView, clickedOnLink link: Any, at charIndex: Int) -> Bool {
    guard
      let (sendto, action) =
        switch link {
        case let link as String: deserializeActionUrl(link)
        case let link as URL: deserializeActionUrl(link.absoluteString)
        default: nil
        }
    else {
      return false
    }

    do {
      try handleLink(sendto, action)
      return true
    } catch {
      print(error.localizedDescription)
      return false
    }
  }

  func receiveOutput(_ fragment: OutputFragment) async {
    switch fragment {
    case .Effect(.Beep):
      await handleBell()
      return
    case .Effect(_):
      return
    case .Hr:
      break
    case .Image:
      break
    case .LineBreak:
      handleBreak()
      willBreak = true
    case .PageBreak:
      break
    case .Text(let text):
      handleBreak()
      textStorage.append(renderText(text, ansiColors))
    }
    textView.scrollRangeToVisible(NSRange(location: textStorage.length, length: 0))

  }

  func sendInput(_ input: String) throws {
    textStorage.append(NSAttributedString(string: "\n" + input, attributes: inputAttrs))
    willBreak = true
    try bridge.send_input(input + "\r\n")
  }

  func handleBell() async {
    let _ = await MainActor.run {
      NSApplication.shared.requestUserAttention(.criticalRequest)
    }
  }

  func handleBreak() {
    if willBreak {
      textStorage.append(NSAttributedString("\n"))
      willBreak = false
    }
  }

  func handleLink(_ sendto: InternalSendTo, _ text: Substring) throws {
    switch sendto {
    case .Input:
      inputField.stringValue = String(text)
    case .World:
      try sendInput(String(text))
    }
  }
}
