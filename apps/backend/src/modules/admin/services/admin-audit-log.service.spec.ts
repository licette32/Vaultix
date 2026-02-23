import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { AdminAuditLogService } from './admin-audit-log.service';
import { AdminAuditLog } from '../entities/admin-audit-log.entity';
import { Repository } from 'typeorm';

describe('AdminAuditLogService', () => {
  let service: AdminAuditLogService;
  let repo: Repository<AdminAuditLog>;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AdminAuditLogService,
        {
          provide: getRepositoryToken(AdminAuditLog),
          useClass: Repository,
        },
      ],
    }).compile();

    service = module.get<AdminAuditLogService>(AdminAuditLogService);
    repo = module.get<Repository<AdminAuditLog>>(getRepositoryToken(AdminAuditLog));
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should create an audit log entry', async () => {
    const dto = {
      actorId: 'admin-1',
      actionType: 'SUSPEND_USER',
      resourceType: 'USER',
      resourceId: 'user-123',
      metadata: { oldStatus: true, newStatus: false },
    };
    const saved = { ...dto, id: 'log-1', createdAt: new Date() };
    jest.spyOn(repo, 'create').mockReturnValue(saved as any);
    jest.spyOn(repo, 'save').mockResolvedValue(saved as any);
    const result = await service.create(dto);
    expect(result).toMatchObject(dto);
  });

  it('should query audit logs with filters', async () => {
    const logs = [
      { id: 'log-1', actorId: 'admin-1', actionType: 'SUSPEND_USER', resourceType: 'USER', resourceId: 'user-123', metadata: {}, createdAt: new Date() },
    ];
    jest.spyOn(repo, 'findAndCount').mockResolvedValue([logs, 1]);
    const result = await service.findAll({ actorId: 'admin-1' });
    expect(result.data.length).toBe(1);
    expect(result.total).toBe(1);
    expect(result.data[0].actorId).toBe('admin-1');
  });
});
