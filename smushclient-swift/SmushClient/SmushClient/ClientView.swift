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
    @State private var output: [TextFragment] = []
    @State private var colors: AnsiColors = defaultAnsiColors()
    
    init(address: String, port: UInt16) {
        self.bridge = RustMudBridge(address, port)
    }
    
    func connect() {
        Task {
            do {
                try await self.bridge.connect()
                while true {
                    let fragment = try await self.bridge.get_output()
                    switch fragment {
                    case .Text(let text):
                        output.append(TextFragment(text))
                    default:
                        break
                    }
                }
            } catch {
                self.errorMessage = error.localizedDescription
            }
        }
    }
    
    var body: some View {
        VStack(alignment: .leading) {
            ScrollView {
                LazyVStack {
                    ForEach(output) { fragment in
                        fragment.render(colors)
                    }
                }
            }
            TextField("", text: $input)
                .onSubmit {
                    do {
                        try self.bridge.send_input(input + "\r\n")
                    } catch {
                        self.errorMessage = error.localizedDescription
                    }
                    input = ""
                }
                .padding(.vertical)
            HStack {
                Button("Connect", action: connect)
                Text(errorMessage).foregroundStyle(.red)
            }
        }
        .padding()
    }
}

#Preview {
    ClientView(address: "82.68.167.69", port: 4242)
}
