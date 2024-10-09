// File: TorModule.kt

package com.tor

import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactMethod
import android.util.Log

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
    override fun connectToTorNetwork(target: String, promise: Promise) {
        try {
            Log.d("TorModule", "Attempting to connect to Tor network")
            val filesDir = reactContext.cacheDir.absolutePath
            Log.d("TorModule", "Cache dir: $filesDir")
            val result = nativeConnectToTorNetwork(target, filesDir)
            Log.d("TorModule", "Tor connection result: $result")
            promise.resolve(result)
        } catch (e: Exception) {
            Log.e("TorModule", "Error connecting to Tor network", e)
            promise.reject("TOR_ERROR", e.message)
        }
    }

    private external fun nativeConnectToTorNetwork(target: String, cacheDir: String): String

    companion object {
        const val NAME = "Tor"
        init {
            try {
                System.loadLibrary("tor")
                Log.d("TorModule", "Tor library loaded successfully")
            } catch (e: UnsatisfiedLinkError) {
                Log.e("TorModule", "Failed to load Tor library", e)
            }
        }
    }
}
