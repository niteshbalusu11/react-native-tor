import { useState, useEffect } from 'react';
import { StyleSheet, View, Text } from 'react-native';
import { multiply, connectToTorNetwork } from 'react-native-tor';

export default function App() {
  const [result, setResult] = useState<number | undefined>();

  useEffect(() => {
    const connect = async () => {
      try {
        const res = await connectToTorNetwork('ifconfig.me');
        console.log(res);
      } catch (error) {
        console.error('Error connecting to Tor network:', error);
      }
    };

    connect();
    multiply(3, 7).then(setResult);
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
