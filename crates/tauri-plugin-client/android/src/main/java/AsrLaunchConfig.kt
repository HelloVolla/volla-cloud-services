package org.holochain.androidserviceruntime.plugin.client

enum class AsrLaunchMode { AUTO, LAUNCHER, SETTINGS }

data class AsrLaunchConfig(
    val packageName: String,
    val mode: AsrLaunchMode = AsrLaunchMode.AUTO,
)
