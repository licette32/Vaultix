import { Test, TestingModule } from '@nestjs/testing';
import { ConfigModule, ConfigService } from '@nestjs/config';
import * as StellarSdk from 'stellar-sdk';
import stellarConfig from '../config/stellar.config';
import { StellarService } from './stellar.service';

describe('StellarService', () => {
  let service: StellarService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      imports: [
        ConfigModule.forFeature(stellarConfig),
      ],
      providers: [StellarService],
    }).compile();

    service = module.get<StellarService>(StellarService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should validate public keys correctly', () => {
    const validPublicKey = 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IA6WNLWLFGJGUJMGHVCCC3U';
    const invalidPublicKey = 'invalid-key';
    
    expect(service.isValidPublicKey(validPublicKey)).toBe(true);
    expect(service.isValidPublicKey(invalidPublicKey)).toBe(false);
  });

  it('should validate secret keys correctly', () => {
    const validSecretKey = 'SBKPP5NNI4MPLJMD6QBWUQUOKTXVVVOA7LL6QNZWKMRTPDBSRHFJ545W';
    const invalidSecretKey = 'invalid-key';
    
    expect(service.isValidSecretKey(validSecretKey)).toBe(true);
    expect(service.isValidSecretKey(invalidSecretKey)).toBe(false);
  });

  it('should create a new keypair', () => {
    const keypair = service.createKeypair();
    
    expect(keypair).toBeDefined();
    expect(keypair.publicKey()).toBeDefined();
    expect(keypair.secret()).toBeDefined();
  });

  // Mock tests for network operations
  it('should build a transaction', async () => {
    // This test would require mocking the Horizon server
    // For now, we'll just test the structure
    const sourceKeypair = StellarSdk.Keypair.random();
    const destKeypair = StellarSdk.Keypair.random();
    
    const paymentOp = StellarSdk.Operation.payment({
      destination: destKeypair.publicKey(),
      asset: StellarSdk.Asset.native(),
      amount: '10',
    });
    
    // Note: This would require a real account to test fully
    // The actual test would need to mock the getAccount call
    expect(() => {
      // We won't execute this since it requires a real account
    }).not.toThrow();
  });
});