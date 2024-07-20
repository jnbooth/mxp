//
//  ContentView.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/17/24.
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        ClientView(address: "82.68.167.69", port: 4242)
    }
}

#Preview {
    ContentView()
}
