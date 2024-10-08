
#ifdef RCT_NEW_ARCH_ENABLED
#import "RNTorSpec.h"

@interface Tor : NSObject <NativeTorSpec>
#else
#import <React/RCTBridgeModule.h>

@interface Tor : NSObject <RCTBridgeModule>
#endif

@end
