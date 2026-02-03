import pandas as pd
import random

class generate_synthetic_csv:
    
    def __init__(self, num_rows: int):
        self.num_rows = num_rows
        self.data = self._generate_data()
        self.df = pd.DataFrame(self.data, columns=['id', 'sequence'])
    
    
    def _generate_data(self):
        data = []
        for i in range(self.num_rows):
            seq_length = 1285
            sequence = ''.join(random.choices(['A', 'T', 'C', 'G'], k=seq_length))
            data.append([f'seq_{i+1}', sequence])
        return data
    
    def to_csv(self, file_path: str):
        self.df.to_csv(file_path, index=False)
        
if __name__ == "__main__":
    generator = generate_synthetic_csv(num_rows=1000)
    generator.to_csv('synthetic_data.csv')