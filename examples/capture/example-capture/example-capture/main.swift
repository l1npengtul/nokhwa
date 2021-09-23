//
//  main.swift
//  example-capture
//
//  Created by pengg on 2021/09/22.
//  Copyright © 2021 l1npengtul-nokhwa. All rights reserved.
//

import Cocoa

func log(_ data: Data) {
    if let message = NSString(data: data, encoding: String.Encoding.utf8.rawValue) {
        print(message)
    }
}


let task = Process()
let bundle = Bundle.main
let rustBinName = bundle.infoDictionary?["RustBinName"] as! String
task.launchPath = bundle.path(forResource: rustBinName, ofType: nil)
task.environment = ["RUST_BACKTRACE": "1"]

let arguments = CommandLine.arguments
var passed_arguments = ""
for argument in arguments {
    if !(argument.starts(with: "/") || argument.starts(with: "-NS")) && argument != "YES" && argument != "NO" {
        passed_arguments += " "
        passed_arguments += argument
    }
}

print(passed_arguments)

let stdOut = Pipe()
let stdErr = Pipe()

stdOut.fileHandleForReading.readabilityHandler = { log($0.availableData) }
stdErr.fileHandleForReading.readabilityHandler = { log($0.availableData) }

task.standardOutput = stdOut
task.standardError = stdErr

task.terminationHandler = { task in
    (task.standardOutput as AnyObject?)?.fileHandleForReading.readabilityHandler = nil
    (task.standardError as AnyObject?)?.fileHandleForReading.readabilityHandler = nil
}

task.launch()
task.waitUntilExit()
