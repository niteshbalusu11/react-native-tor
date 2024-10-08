import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface Spec extends TurboModule {
  multiply(a: number, b: number): Promise<number>;
  connectToTorNetwork(target: string): Promise<string>;
}

export default TurboModuleRegistry.getEnforcing<Spec>('Tor');
