package com.tor

import com.facebook.react.bridge.ReactApplicationContext

abstract class TorSpec internal constructor(context: ReactApplicationContext) :
  NativeTorSpec(context) {
}
