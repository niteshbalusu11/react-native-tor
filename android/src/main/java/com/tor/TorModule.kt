package com.tor

import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactMethod
import com.facebook.react.bridge.Promise

class TorModule internal constructor(context: ReactApplicationContext) : TorSpec(context) {
    override fun getName(): String {
        return NAME
    }

    @ReactMethod
    override fun multiply(a: Double, b: Double, promise: Promise) {
        promise.resolve(a * b)
    }

    @ReactMethod
    fun myTorMethod(promise: Promise) {
        try {
            val result = nativeMyTorMethod()
            promise.resolve(result)
        } catch (e: Exception) {
            promise.reject("TOR_ERROR", e.message)
        }
    }

    private external fun nativeMyTorMethod(): String

    companion object {
        const val NAME = "Tor"

        init {
            System.loadLibrary("tor")
        }
    }
}
