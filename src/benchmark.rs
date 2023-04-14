
pub enum BenchmarkStatus {
    Paused,
    Running,
    Finished,
}

pub struct Benchmark {
    elapsed_time: f64,
    number_iterations: i32,
    benchmark_iterations: i32,
    pub status: BenchmarkStatus,
}

impl Benchmark {
    pub fn new(benchmark_iterations: i32) -> Benchmark {
        Benchmark {
            elapsed_time: 0.,
            number_iterations: 0,
            benchmark_iterations,
            status: BenchmarkStatus::Paused,
        }
    }

    pub fn start(&mut self) {
        println!("Running benchmark...");
        self.status = BenchmarkStatus::Running;
    }

    pub fn increase_elapsed_time(&mut self, elapsed_time: f64) {
        self.elapsed_time += elapsed_time;
        self.number_iterations += 1;

        if self.number_iterations >= self.benchmark_iterations {
            self.status = BenchmarkStatus::Finished;
            println!("Completed benchmark");
            println!("Result from benchmark: Time={}s,Iterations={}, Average={}s", self.elapsed_time, self.number_iterations, self.get_average_time())
        }
    }

    pub fn get_average_time(&self) -> f64 {
        self.elapsed_time / self.number_iterations as f64
    }
}

