// File: TorModule.kt

package com.tor

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactMethod
import com.facebook.react.bridge.Promise

class TorModule internal constructor(context: ReactApplicationContext) : TorSpec(context) {
    private val reactContext: ReactApplicationContext = context

    override fun getName(): String {
        return NAME
    }

    @ReactMethod
    override fun multiply(a: Double, b: Double, promise: Promise) {
        promise.resolve(a * b)
    }

    @ReactMethod
    fun connectToTorNetwork(target: String, promise: Promise) {
        try {
            val cacheDir = reactContext.cacheDir.absolutePath
            val result = nativeConnectToTorNetwork(target, cacheDir)
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("TOR_ERROR", e.message)
        }
    }

    private external fun nativeConnectToTorNetwork(target: String, cacheDir: String): String

    companion object {
        const val NAME = "Tor"
        init {
            System.loadLibrary("tor")
        }
    }
}
