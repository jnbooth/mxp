//
//  actions.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/20/24.
//

import Foundation
import SwiftUI

public let actionScheme = "smushclient"
let inputPrefix = actionScheme + "://input?text="
let worldPrefix = actionScheme + "://send?text="

public enum InternalSendTo {
    case Input;
    case World;
}

public func getAction(_ action: RustString, _ text: String) -> String {
    return action.toString().replacing("&text;", with: text)
}

public func serializeActionUrl(_ sendto: SendTo, _ action: String) -> URL? {
    switch sendto {
    case .Input:
        return URL(string: inputPrefix + action)
    case .Internet:
        return URL(string: action)
    case .World:
        return URL(string: worldPrefix + action)
    }
}

public func deserializeActionUrl(_ url: URL) -> (InternalSendTo, String)? {
    guard
        let baseURL = url.host(percentEncoded: false),
        let query = url.query(percentEncoded: false),
        let splitAfter = query.firstIndex(of: "="),
        let action = query.suffix(from: query.index(splitAfter, offsetBy: 1)).removingPercentEncoding
    else {
        return nil
    }
    switch baseURL {
    case "input":
        return (.Input, action)
    case "send":
        return (.World, action)
    default:
        return nil
    }
}
