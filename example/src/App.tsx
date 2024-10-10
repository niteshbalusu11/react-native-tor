import { useState, useEffect } from 'react';
import { StyleSheet, View, Text } from 'react-native';
import { connectToTorNetwork, multiply } from 'react-native-lnd-tor';
import axios from 'axios';

export default function App() {
  const [torIp, setTorIp] = useState<string | undefined>();

  useEffect(() => {
    const connect = async () => {
      try {
        const mul = await multiply(8, 4);

        console.log('multiply res', mul);
        // Initialize Tor proxy
        const res = await connectToTorNetwork('ifconfig.me');
        console.log(res);

        // Make a request to get the public IP through the Tor network
        const response = await axios.get('https://ifconfig.me', {
          proxy: {
            host: '127.0.0.1',
            port: 9050, // This is the port where the SOCKS proxy is running
            protocol: 'socks5', // Important: use SOCKS5 protocol
          },
        });

        // Log the IP address returned by the request
        console.log('IP from Tor network:', response.data);
        setTorIp(response.data);
      } catch (error) {
        console.error(
          'Error connecting to Tor network or making request:',
          error
        );
      }
    };

    connect();
  }, []);

  return (
    <View style={styles.container}>
      <Text>Tor Network IP: {torIp}</Text>
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
