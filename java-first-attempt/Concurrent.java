public class Concurrent {
  static final int NUM_THREADS = 10;
  static final int NUM_CALLS = 1_000_000;

  volatile boolean started = false;

  volatile int counter = 0;

  public static void main(final String[] args) throws Exception {
    final Concurrent c = new Concurrent();

    final Thread[] threads = new Thread[NUM_THREADS];
    for (int i = 0; i < NUM_THREADS; i++) {
      threads[i] = new Thread(new Runnable() {
        public void run() {
          while (!c.started);
          for (int i = 0; i < NUM_CALLS; i++) c.counter++;
        }
      });
      threads[i].start();
    }

    c.started = true;
    for (int i = 0; i < NUM_THREADS; i++) {
      threads[i].join();
    }

    System.out.println(c.counter);
    System.exit(0);
  }
}
