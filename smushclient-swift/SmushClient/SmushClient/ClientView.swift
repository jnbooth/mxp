//
//  ClientView.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/19/24.
//

import SwiftUI

struct ClientView: View {
    private var bridge: RustMudBridge
    @State private var input: String = ""
    @State private var errorMessage: String = ""
    @State private var lines: [MudLine] = []
    @State private var line: AttributedString = AttributedString()
    @State private var colors: AnsiColors = defaultAnsiColors()
    @State private var willBreak: Bool = false
    @State private var ansiColors: AnsiColors = defaultAnsiColors()

    init(address: String, port: UInt16) {
        bridge = RustMudBridge(address, port)
    }

    func connect() async {
        do {
            try await bridge.connect()
            while true {
                let fragment = try await bridge.get_output()
                await self.receiveOutput(fragment)
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }
    
    func receiveOutput(_ fragment: OutputFragment) async {
        switch fragment {
        case .Effect(.Beep):
            await handleBell()
        case .LineBreak:
            handleBreak()
            willBreak = true
        case .Text(let text):
            handleBreak()
            line += renderText(text, colors)
        default:
            break
        }
    }
    
    func sendInput(_ input: String) {
        lines.append(MudLine.init(line))
        line = AttributedString(input)
        line.foregroundColor = .gray
        willBreak = true
        do {
            try bridge.send_input(input + "\r\n")
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func handleBell() async {
        let _ = await MainActor.run {
            NSApplication.shared.requestUserAttention(.criticalRequest)
        }
    }
    
    func handleBreak() {
        if willBreak {
            line += AttributedString("\n")
            willBreak = false
        }
    }
    
    func handleInput() {
        sendInput(input)
        input = ""
    }
    
    func handleLink(_ url: URL) {
        guard let (sendto, action) = deserializeActionUrl(url) else {
            return
        }
        switch sendto {
        case .Input:
            input = action
        case .World:
            sendInput(action)
        }
    }

    var body: some View {
        VStack(alignment: .leading) {
            ScrollView {
                ScrollViewReader { scrollView in
                    LazyVStack(alignment: .leading) {
                        ForEach(lines) { line in
                            Text(line.text).textSelection(.enabled)
                        }
                        Text(line).id("line").textSelection(.enabled)
                    }.onChange(of: line) {
                        scrollView.scrollTo("line")
                    }
                }
            }
            TextField("", text: $input)
                .onSubmit(handleInput)
                .padding(.bottom)
            Text(errorMessage).foregroundStyle(.red)
        }
        .padding()
        .monospaced().onOpenURL(perform: handleLink)
        .task {
            await connect()
        }
    }
}

#Preview {
    ClientView(address: "82.68.167.69", port: 4242)
}
