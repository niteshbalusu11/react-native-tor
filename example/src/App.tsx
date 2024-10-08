import { useState, useEffect } from 'react';
import { StyleSheet, View, Text } from 'react-native';
import { multiply, startTor } from 'react-native-tor';

export default function App() {
  const [result, setResult] = useState<number | undefined>();

  startTor()
    .then((res: any) => console.log(res))
    .catch((err: any) => console.error(err));

  useEffect(() => {
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
