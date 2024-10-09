import { useState, useEffect } from 'react';
import { StyleSheet, View, Text } from 'react-native';
import { multiply, connectToTorNetwork } from 'react-native-lnd-tor';

export default function App() {
  const [result, setResult] = useState<number | undefined>();

  useEffect(() => {
    const connect = async () => {
      try {
        const res = await connectToTorNetwork('ifconfig.me');
        console.log(res);
        const mul = await multiply(4, 8);
        console.log('multiply result', mul);
        setResult(mul);
      } catch (error) {
        console.error('Error connecting to Tor network:', error);
      }
    };

    connect();
  }, []);

  return (
    <View style={styles.container}>
      <Text>Result: {result}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  box: {
    width: 60,
    height: 60,
    marginVertical: 20,
  },
});
