public enum InternalSendTo {
    case Input;
    case World;
}

public func getAction(_ action: RustString, _ text: String) -> String {
    return action.toString().replacing("&text;", with: text)
}

public func serializeActionUrl(_ sendto: SendTo, _ action: String) -> String {
    switch sendto {
    case .Input:
        return "input:" + action
    case .Internet:
        return action
    case .World:
        return "send:" + action
    }
}

public func deserializeActionUrl(_ url: String) -> (InternalSendTo, Substring)? {
    let components = url.split(separator: ":", maxSplits: 1)
    if components.count < 2 {
        return nil
    }
    switch components[0] {
    case "input":
        return (.Input, components[1])
    case "send":
        return (.World, components[1])
    default:
        return nil
    }
}
