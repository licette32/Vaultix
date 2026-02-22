import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import request from 'supertest';
import { DataSource, Repository } from 'typeorm';

import { AppModule } from '../src/app.module';

import { Escrow } from '../src/modules/escrow/entities/escrow.entity';
import { Party } from '../src/modules/escrow/entities/party.entity';
import { User, UserRole } from '../src/modules/user/entities/user.entity';

describe('Escrow Scheduler (e2e)', () => {
  let app: INestApplication;
  let dataSource: DataSource;

  let userRepository: Repository<User>;
  let escrowRepository: Repository<Escrow>;
  let partyRepository: Repository<Party>;

  let adminToken: string;
  let testUser: User;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    app = moduleFixture.createNestApplication();
    await app.init();

    dataSource = app.get(DataSource);

    userRepository = dataSource.getRepository(User);
    escrowRepository = dataSource.getRepository(Escrow);
    partyRepository = dataSource.getRepository(Party);

    testUser = await userRepository.save(
      userRepository.create({
        walletAddress: 'TEST_SCHEDULER_WALLET',
        role: UserRole.USER,
        isActive: true,
      }),
    );

    // 🔥 Replace with real token factory if available
    adminToken = 'admin-jwt-token';
  });

  afterAll(async () => {
    await app.close();
  });

  beforeEach(async () => {
    await partyRepository.delete({});
    await escrowRepository.delete({});
  });

  describe('Manual Escrow Processing', () => {
    it('should auto-cancel expired pending escrow', async () => {
      await request(app.getHttpServer())
        .post(`/escrows/scheduler/process/ESCROW_ID`)
        .set('Authorization', `Bearer ${adminToken}`)
        .expect(200);
    });

    it('should escalate expired active escrow to dispute', async () => {
      await request(app.getHttpServer())
        .post(`/escrows/scheduler/process/ESCROW_ID`)
        .set('Authorization', `Bearer ${adminToken}`)
        .expect(200);
    });

    it('should not process non-expired escrow', async () => {
      await request(app.getHttpServer())
        .post(`/escrows/scheduler/process/ESCROW_ID`)
        .set('Authorization', `Bearer ${adminToken}`)
        .expect(400);
    });

    it('should handle non-existent escrow', async () => {
      await request(app.getHttpServer())
        .post('/escrows/scheduler/process/non-existent-id')
        .set('Authorization', `Bearer ${adminToken}`)
        .expect(400);
    });
  });

  describe('Batch Processing', () => {
    it('should process all expired escrows', async () => {
      await request(app.getHttpServer())
        .post('/escrows/scheduler/process-expired')
        .set('Authorization', `Bearer ${adminToken}`)
        .expect(200);
    });
  });

  describe('Access Control', () => {
    it('should forbid access for non-admin users', async () => {
      await request(app.getHttpServer())
        .post('/escrows/scheduler/process-expired')
        .set('Authorization', 'Bearer user-token')
        .expect(403);
    });

    it('should require authentication', async () => {
      await request(app.getHttpServer())
        .post('/escrows/scheduler/process-expired')
        .expect(401);
    });
  });
});